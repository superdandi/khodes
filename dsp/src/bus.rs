//! Bus global de efectos, post-mezcla de voces.
//!
//! Réplica del orden documentado en `docs/VOICE_SPEC.md` §4:
//! `drive → brightness → bass → treble → chorus → tremolo → reverb → master`.
//!
//! El bus es mono a la entrada (suma de voces) y estéreo a la salida. El
//! estéreo lo generan el chorus (dos líneas con LFO desfasado) y la reverb
//! (dos IR decorrelacionadas). Con chorus y reverb a cero, ambos canales son
//! idénticos: el caso A/B contra Keys sigue siendo mono.
//!
//! No hay asignaciones en `process`: IR, delay lines, historial FFT y buffers
//! temporales se reservan en `prepare`. Ver `docs/adr/0002-port-keys-rhodes-voice.md`.

use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::params::Params;
use crate::voice::flush_denormal;

/// Tamaño de partición de la convolución. Latencia del efecto: 256 frames.
const PARTITION: usize = 256;
/// FFT de overlap-save: dos particiones.
const FFT_SIZE: usize = PARTITION * 2;
/// Duración de la IR de reverb (`VOICE_SPEC` §4.6).
const IR_SECONDS: f64 = 2.5;
/// Decaimiento exponencial de la IR.
const IR_DECAY_S: f64 = 0.6;
/// Delay base del chorus (`rhodes.ts`: `CHORUS_DELAY_BASE`).
const CHORUS_BASE_S: f64 = 0.022;
/// Excursión fija del LFO del chorus (`rhodes.ts`: `CHORUS_LFO_DEPTH`).
const CHORUS_LFO_S: f64 = 0.004;
/// Capacidad máxima de la línea de delay del chorus.
const CHORUS_MAX_S: f64 = 0.05;
/// Cantidad en que CC1 incrementa la profundidad de tremolo (`MIDI.md` §6).
const MOD_AMOUNT: f64 = 0.5;
/// Sub-bloque para recalcular coeficientes de filtros y suavizado.
const SUB_BLOCK: usize = 32;
/// Umbral del limitador de seguridad; por encima, compresión suave.
const LIMIT_THRESHOLD: f64 = 0.95;

/// Filtro biquad RBJ con estado directo forma I.
#[derive(Clone, Copy, Debug)]
struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }
}

impl Biquad {
    fn set_coefficients(&mut self, b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) {
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn lowpass(sample_rate: f64, freq_hz: f64, q: f64) -> Self {
        let w0 = std::f64::consts::TAU * freq_hz.clamp(10.0, sample_rate * 0.45) / sample_rate;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q.max(0.0001));
        let b0 = (1.0 - cos) * 0.5;
        let b1 = 1.0 - cos;
        let b2 = b0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;
        let mut f = Self::default();
        f.set_coefficients(b0, b1, b2, a0, a1, a2);
        f
    }

    fn shelf(sample_rate: f64, freq_hz: f64, gain_db: f64, high: bool) -> Self {
        let w0 = std::f64::consts::TAU * freq_hz.clamp(10.0, sample_rate * 0.45) / sample_rate;
        let (sin, cos) = w0.sin_cos();
        let a = 10_f64.powf(gain_db / 40.0);
        let sqrt_a = a.sqrt();
        // RBJ con S = 1, igual que el slope de Web Audio.
        let alpha = sin * 0.5 * 2.0_f64.sqrt();

        let (b0, b1, b2, a0, a1, a2) = if high {
            (
                a * ((a + 1.0) + (a - 1.0) * cos + 2.0 * sqrt_a * alpha),
                -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
                a * ((a + 1.0) + (a - 1.0) * cos - 2.0 * sqrt_a * alpha),
                (a + 1.0) - (a - 1.0) * cos + 2.0 * sqrt_a * alpha,
                2.0 * ((a - 1.0) - (a + 1.0) * cos),
                (a + 1.0) - (a - 1.0) * cos - 2.0 * sqrt_a * alpha,
            )
        } else {
            (
                a * ((a + 1.0) - (a - 1.0) * cos + 2.0 * sqrt_a * alpha),
                2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
                a * ((a + 1.0) - (a - 1.0) * cos - 2.0 * sqrt_a * alpha),
                (a + 1.0) + (a - 1.0) * cos + 2.0 * sqrt_a * alpha,
                -2.0 * ((a - 1.0) + (a + 1.0) * cos),
                (a + 1.0) + (a - 1.0) * cos - 2.0 * sqrt_a * alpha,
            )
        };
        let mut f = Self::default();
        f.set_coefficients(b0, b1, b2, a0, a1, a2);
        f
    }

    #[inline]
    fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// Valor con suavizado exponencial de ~20 ms.
#[derive(Clone, Copy, Debug)]
struct Smooth {
    current: f64,
    target: f64,
}

impl Smooth {
    const fn new(value: f64) -> Self {
        Self {
            current: value,
            target: value,
        }
    }

    fn set(&mut self, value: f64) {
        self.target = value;
    }

    #[inline]
    fn tick(&mut self, coeff: f64) -> f64 {
        self.current += coeff * (self.target - self.current);
        self.current
    }
}

/// Parámetros suavizados del bus.
#[derive(Clone, Copy, Debug)]
struct SmoothedParams {
    brightness_hz: Smooth,
    bass_db: Smooth,
    treble_db: Smooth,
    drive: Smooth,
    chorus_rate: Smooth,
    chorus_depth: Smooth,
    tremolo_rate: Smooth,
    tremolo_depth: Smooth,
    reverb: Smooth,
    volume: Smooth,
    mod_wheel: Smooth,
}

impl SmoothedParams {
    fn from(params: &Params) -> Self {
        Self {
            brightness_hz: Smooth::new(params.brightness_hz),
            bass_db: Smooth::new(params.bass_db),
            treble_db: Smooth::new(params.treble_db),
            drive: Smooth::new(params.drive),
            chorus_rate: Smooth::new(params.chorus_rate_hz),
            chorus_depth: Smooth::new(params.chorus_depth),
            tremolo_rate: Smooth::new(params.tremolo_rate_hz),
            tremolo_depth: Smooth::new(params.tremolo_depth),
            reverb: Smooth::new(params.reverb),
            volume: Smooth::new(params.volume),
            mod_wheel: Smooth::new(0.0),
        }
    }

    fn set_targets(&mut self, params: &Params, mod_wheel: f64) {
        self.brightness_hz.set(params.brightness_hz);
        self.bass_db.set(params.bass_db);
        self.treble_db.set(params.treble_db);
        self.drive.set(params.drive);
        self.chorus_rate.set(params.chorus_rate_hz);
        self.chorus_depth.set(params.chorus_depth);
        self.tremolo_rate.set(params.tremolo_rate_hz);
        self.tremolo_depth.set(params.tremolo_depth);
        self.reverb.set(params.reverb);
        self.volume.set(params.volume);
        self.mod_wheel.set(mod_wheel);
    }
}

/// Estado de una de las dos líneas de delay del chorus.
#[derive(Clone, Debug)]
struct ChorusLine {
    buffer: Vec<f64>,
    write: usize,
    phase: f64,
}

impl Default for ChorusLine {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            write: 0,
            phase: 0.0,
        }
    }
}

/// Bus global. La memoria se reserva en `prepare`; `process` no asigna.
pub struct Bus {
    sample_rate: f64,
    smooth_coeff: f64,
    smooth: SmoothedParams,

    brightness: Biquad,
    bass: Biquad,
    treble: Biquad,

    chorus: [ChorusLine; 2],

    tremolo_phase: f64,

    /// IR en dominio de frecuencia, por canal y partición.
    ir: [Vec<Vec<Complex<f64>>>; 2],
    /// Historial de bloques en frecuencia, por canal y partición.
    x_history: [Vec<Vec<Complex<f64>>>; 2],
    /// Bloque anterior en tiempo (overlap-save), por canal.
    time_history: [[f64; PARTITION]; 2],
    /// Entrada pendiente de reverb, por canal.
    rev_in: [[f64; PARTITION]; 2],
    /// Salida disponible de reverb, por canal.
    rev_out: [[f64; PARTITION]; 2],
    rev_in_len: usize,
    rev_out_pos: usize,

    fft_input: Vec<Complex<f64>>,
    fft_output: Vec<Complex<f64>>,
    accum: Vec<Complex<f64>>,
    ifft_output: Vec<Complex<f64>>,

    fft: Arc<dyn Fft<f64>>,
    ifft: Arc<dyn Fft<f64>>,

    /// Buffer temporal mono entre etapas.
    mono: Vec<f64>,
    /// Copia de entrada del drive, para no asignar en `process`.
    drive_input: Vec<f64>,
    /// Última muestra vista por el oversampling del drive.
    drive_prev: f64,
    /// Latencia del oversampling del drive en muestras.
    drive_latency: usize,
}

impl Default for Bus {
    fn default() -> Self {
        let mut planner = FftPlanner::<f64>::new();
        Self {
            sample_rate: 48_000.0,
            smooth_coeff: 0.0,
            smooth: SmoothedParams::from(&Params::default()),
            brightness: Biquad::default(),
            bass: Biquad::default(),
            treble: Biquad::default(),
            chorus: std::array::from_fn(|_| ChorusLine::default()),
            tremolo_phase: 0.0,
            ir: [Vec::new(), Vec::new()],
            x_history: [Vec::new(), Vec::new()],
            time_history: [[0.0; PARTITION]; 2],
            rev_in: [[0.0; PARTITION]; 2],
            rev_out: [[0.0; PARTITION]; 2],
            rev_in_len: 0,
            rev_out_pos: 0,
            fft_input: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            fft_output: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            accum: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            ifft_output: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            fft: planner.plan_fft_forward(FFT_SIZE),
            ifft: planner.plan_fft_inverse(FFT_SIZE),
            mono: Vec::new(),
            drive_input: Vec::new(),
            drive_prev: 0.0,
            drive_latency: 1,
        }
    }
}

impl Bus {
    /// Reserva IR, delays y buffers. Puede asignar; no se llama en `process`.
    pub fn prepare(&mut self, sample_rate: f64, max_block_size: usize) {
        let sr = sample_rate.max(1.0);
        self.sample_rate = sr;
        self.smooth_coeff = 1.0 - (-f64::from(SUB_BLOCK as u32) / (0.02 * sr)).exp();
        self.mono.resize(max_block_size.max(1), 0.0);
        self.drive_input.resize(max_block_size.max(1), 0.0);

        let chorus_len = (CHORUS_MAX_S * sr).ceil() as usize;
        for (channel, line) in self.chorus.iter_mut().enumerate() {
            line.buffer.resize(chorus_len, 0.0);
            line.write = 0;
            line.phase = if channel == 0 {
                0.0
            } else {
                std::f64::consts::FRAC_PI_2
            };
        }
        self.tremolo_phase = 0.0;

        self.build_ir(sr);
        self.reset();
    }

    /// Genera dos IR deterministas y calcula sus FFT por particiones.
    fn build_ir(&mut self, sample_rate: f64) {
        let ir_len = (IR_SECONDS * sample_rate).ceil() as usize;
        let partitions = ir_len.div_ceil(PARTITION);
        let mut scratch = vec![Complex::new(0.0, 0.0); FFT_SIZE];

        for channel in 0..2 {
            let seed = if channel == 0 {
                0x9E37_79B9_7F4A_7C15_u64
            } else {
                0xD1B5_4A32_D192_ED03_u64
            };
            let mut rng = seed;
            let mut samples = vec![0.0; partitions * PARTITION];
            for (i, sample) in samples.iter_mut().enumerate().take(ir_len) {
                let decay = (-(i as f64) / (IR_DECAY_S * sample_rate)).exp();
                // Fórmula: next_random del motor, replicada sin exponer PRNG aquí.
                rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = rng;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^= z >> 31;
                let noise = ((z >> 11) as f64 / (1_u64 << 53) as f64) * 2.0 - 1.0;
                *sample = noise * decay;
            }

            self.ir[channel].clear();
            self.x_history[channel].clear();
            for p in 0..partitions {
                scratch.fill(Complex::new(0.0, 0.0));
                for i in 0..PARTITION {
                    scratch[i] = Complex::new(samples[p * PARTITION + i], 0.0);
                }
                let mut spectrum = scratch.clone();
                self.fft.process(&mut spectrum);
                self.ir[channel].push(spectrum);
                self.x_history[channel].push(vec![Complex::new(0.0, 0.0); FFT_SIZE]);
            }
        }
    }

    /// Limpia el estado transitorio sin liberar memoria.
    pub fn reset(&mut self) {
        self.smooth = SmoothedParams::from(&Params::default());
        for filter in [&mut self.brightness, &mut self.bass, &mut self.treble] {
            *filter = Biquad::default();
        }
        for line in &mut self.chorus {
            line.buffer.fill(0.0);
            line.write = 0;
        }
        self.tremolo_phase = 0.0;
        self.drive_prev = 0.0;
        for channel in 0..2 {
            self.time_history[channel].fill(0.0);
            self.rev_in[channel].fill(0.0);
            self.rev_out[channel].fill(0.0);
            for spectrum in &mut self.x_history[channel] {
                spectrum.fill(Complex::new(0.0, 0.0));
            }
        }
        self.rev_in_len = 0;
        self.rev_out_pos = 0;
    }

    /// Latencia introducida por el oversampling del drive, en muestras.
    #[must_use]
    pub const fn reported_latency_samples(&self) -> usize {
        self.drive_latency
    }

    /// Aplica drive tanh con oversampling 4x a un sub-bloque. Con `drive = 0`,
    /// bypass exacto, pero se actualiza la muestra anterior para que el
    /// encendido del drive no produzca un salto.
    fn drive_block(&mut self, amount: f64, start: usize, end: usize) {
        if amount <= 1e-9 {
            if end > start {
                self.drive_prev = self.mono[end - 1];
            }
            return;
        }
        self.drive_input[start..end].copy_from_slice(&self.mono[start..end]);
        let k = 1.0 + amount * 10.0;
        let tanh_k = k.tanh();
        let mut previous = self.drive_prev;
        for i in start..end {
            let current = self.drive_input[i];
            let delta = current - previous;
            let u1 = previous + delta * 0.25;
            let u2 = previous + delta * 0.5;
            let u3 = previous + delta * 0.75;
            let y0 = (k * previous).tanh() / tanh_k;
            let y1 = (k * u1).tanh() / tanh_k;
            let y2 = (k * u2).tanh() / tanh_k;
            let y3 = (k * u3).tanh() / tanh_k;
            self.mono[i] = (y0 + y1 + y2 + y3) * 0.25;
            previous = current;
        }
        self.drive_prev = previous;
    }

    /// Convolución por particiones con overlap-save para un canal.
    fn convolve(&mut self, channel: usize, output: &mut [f64; PARTITION]) {
        let partitions = self.ir[channel].len();
        if partitions == 0 {
            output.fill(0.0);
            return;
        }

        for i in 0..PARTITION {
            self.fft_input[i] = Complex::new(self.time_history[channel][i], 0.0);
            self.fft_input[PARTITION + i] = Complex::new(self.rev_in[channel][i], 0.0);
        }
        self.fft_output.copy_from_slice(&self.fft_input);
        self.fft.process(&mut self.fft_output);

        self.x_history[channel].rotate_right(1);
        self.x_history[channel][0].copy_from_slice(&self.fft_output);

        self.accum.fill(Complex::new(0.0, 0.0));
        for (ir_part, x_part) in self.ir[channel].iter().zip(self.x_history[channel].iter()) {
            for i in 0..FFT_SIZE {
                self.accum[i] += ir_part[i] * x_part[i];
            }
        }

        self.ifft_output.copy_from_slice(&self.accum);
        self.ifft.process(&mut self.ifft_output);
        let scale = 1.0 / FFT_SIZE as f64;
        for (i, sample) in output.iter_mut().enumerate() {
            *sample = self.ifft_output[PARTITION + i].re * scale;
        }
        self.time_history[channel].copy_from_slice(&self.rev_in[channel]);
    }

    /// Procesa un bloque estéreo. `left` y `right` deben tener la misma longitud.
    /// No asigna memoria ni redimensiona buffers.
    pub fn process(
        &mut self,
        left: &mut [f64],
        right: &mut [f64],
        params: &Params,
        mod_wheel: f64,
    ) {
        let frames = left.len().min(right.len());
        if frames == 0 || frames > self.mono.len() {
            return;
        }
        let sr = self.sample_rate;
        self.smooth.set_targets(params, mod_wheel);

        // Entrada mono: suma de los canales del motor (que hoy es mono en ambos).
        for i in 0..frames {
            self.mono[i] = (left[i] + right[i]) * 0.5;
        }

        let two_pi = std::f64::consts::TAU;
        let phase_inc = |hz: f64| two_pi * hz.max(0.0) / sr;

        let mut offset = 0;
        while offset < frames {
            let end = (offset + SUB_BLOCK).min(frames);

            // El drive se suaviza como el resto de parámetros y se aplica al
            // sub-bloque antes de los filtros. Llamarlo fuera de este bucle
            // usaba el valor previo al `reset` y lo dejaba inactivo.
            let drive = self.smooth.drive.tick(self.smooth_coeff).max(0.0);
            self.drive_block(drive, offset, end);

            let brightness = self.smooth.brightness_hz.tick(self.smooth_coeff);
            let bass_db = self.smooth.bass_db.tick(self.smooth_coeff);
            let treble_db = self.smooth.treble_db.tick(self.smooth_coeff);
            let chorus_rate = self.smooth.chorus_rate.tick(self.smooth_coeff);
            let chorus_depth = self
                .smooth
                .chorus_depth
                .tick(self.smooth_coeff)
                .clamp(0.0, 1.0);
            let tremolo_rate = self.smooth.tremolo_rate.tick(self.smooth_coeff);
            let tremolo_depth = self
                .smooth
                .tremolo_depth
                .tick(self.smooth_coeff)
                .clamp(0.0, 1.0);
            let mod_amount = self
                .smooth
                .mod_wheel
                .tick(self.smooth_coeff)
                .clamp(0.0, 1.0);
            let reverb_mix = self.smooth.reverb.tick(self.smooth_coeff).clamp(0.0, 1.0);
            let volume = self.smooth.volume.tick(self.smooth_coeff).max(0.0);

            self.brightness = Biquad::lowpass(sr, brightness, 1.0);
            self.bass = Biquad::shelf(sr, 250.0, bass_db, false);
            self.treble = Biquad::shelf(sr, 4_000.0, treble_db, true);

            let c_inc = phase_inc(chorus_rate);
            let t_inc = phase_inc(tremolo_rate);
            let wet_depth = (tremolo_depth + mod_amount * MOD_AMOUNT).clamp(0.0, 1.0);
            let need_reverb = reverb_mix > 1e-6;

            for i in offset..end {
                // Drive → brightness → bass → treble.
                let mut sample = self.mono[i];
                sample = self.brightness.process(sample);
                sample = self.bass.process(sample);
                sample = self.treble.process(sample);

                // Chorus: dos líneas desfasadas. Con depth 0, ambas son el dry.
                let mut stereo = [sample, sample];
                if chorus_depth > 1e-6 {
                    for (channel, line) in self.chorus.iter_mut().enumerate() {
                        let delay_s = CHORUS_BASE_S + CHORUS_LFO_S * line.phase.sin();
                        let delay_frames = (delay_s * sr) as usize;
                        let read =
                            (line.write + line.buffer.len() - delay_frames) % line.buffer.len();
                        let delayed = line.buffer[read];
                        stereo[channel] = sample + chorus_depth * delayed;
                        line.buffer[line.write] = sample;
                        line.write = (line.write + 1) % line.buffer.len();
                        line.phase = (line.phase + c_inc).rem_euclid(two_pi);
                    }
                }

                // Tremolo: LFO senoidal sumado a ganancia base 1.
                let tremolo = 1.0 + wet_depth * self.tremolo_phase.sin();
                self.tremolo_phase = (self.tremolo_phase + t_inc).rem_euclid(two_pi);
                stereo[0] *= tremolo;
                stereo[1] *= tremolo;

                // Reverb: la entrada se acumula en bloques de PARTITION y la
                // salida se lee con el desfase propio del efecto.
                let mut wet = [0.0, 0.0];
                if need_reverb {
                    wet[0] = self.rev_out[0][self.rev_out_pos];
                    wet[1] = self.rev_out[1][self.rev_out_pos];
                }
                self.rev_out_pos = (self.rev_out_pos + 1) % PARTITION;
                if need_reverb {
                    for (channel, sample) in stereo.iter().enumerate() {
                        self.rev_in[channel][self.rev_in_len] = *sample;
                    }
                    self.rev_in_len += 1;
                    if self.rev_in_len == PARTITION {
                        let mut out_left = [0.0; PARTITION];
                        let mut out_right = [0.0; PARTITION];
                        self.convolve(0, &mut out_left);
                        self.convolve(1, &mut out_right);
                        self.rev_out[0] = out_left;
                        self.rev_out[1] = out_right;
                        self.rev_in_len = 0;
                    }
                    stereo[0] += reverb_mix * wet[0];
                    stereo[1] += reverb_mix * wet[1];
                }

                // Master + limitador de seguridad + flush de denormales.
                for (channel, sample) in stereo.iter().enumerate() {
                    let mut out = *sample * volume;
                    let abs = out.abs();
                    if abs > LIMIT_THRESHOLD {
                        let excess = (abs - LIMIT_THRESHOLD) / (1.0 - LIMIT_THRESHOLD);
                        out = out.signum()
                            * (LIMIT_THRESHOLD + (1.0 - LIMIT_THRESHOLD) * excess.tanh());
                    }
                    if channel == 0 {
                        left[i] = flush_denormal(out);
                    } else {
                        right[i] = flush_denormal(out);
                    }
                }
            }

            offset = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bus() -> Bus {
        let mut bus = Bus::default();
        bus.prepare(48_000.0, 512);
        bus
    }

    fn render(bytes: &[f64], params: &Params) -> (Vec<f64>, Vec<f64>) {
        // El bus descarta bloques mayores que su capacidad; prepararlo con
        // 512 frames hacía que los tests de 1024/2048 frames fueran un no-op.
        let mut bus = Bus::default();
        bus.prepare(48_000.0, bytes.len());
        let mut l = bytes.to_vec();
        let mut r = bytes.to_vec();
        bus.process(&mut l, &mut r, params, 0.0);
        (l, r)
    }

    fn impulse(len: usize) -> Vec<f64> {
        let mut v = vec![0.0; len];
        v[0] = 1.0;
        v
    }

    #[test]
    fn neutral_chain_is_not_silent() {
        let params = Params::default();
        let (l, _) = render(&impulse(512), &params);
        assert!(l.iter().all(|x| x.is_finite()));
        assert!(l.iter().any(|x| *x != 0.0));
    }

    #[test]
    fn drive_zero_is_exact_bypass_of_drive_stage() {
        let mut bus = bus();
        let input = vec![0.25; 128];
        let mut l = input.clone();
        let mut r = input.clone();
        // Con todo neutro salvo el master a 1, la salida debe ser lineal.
        let params = Params {
            volume: 1.0,
            brightness_hz: 20_000.0,
            ..Params::default()
        };
        bus.process(&mut l, &mut r, &params, 0.0);
        // El filtro atenúa transitorios; comprobamos que no hay saturación.
        let peak = l.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        assert!(peak <= 1.0);
    }

    #[test]
    fn drive_changes_the_waveform() {
        let input = vec![0.8; 256];
        let clean = render(&input, &Params::default()).0;
        let dirty = render(
            &input,
            &Params {
                drive: 0.8,
                ..Params::default()
            },
        )
        .0;
        let delta: f64 = clean
            .iter()
            .zip(dirty.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(delta > 0.0);
    }

    #[test]
    fn brightness_sweep_has_no_discontinuity() {
        let input = vec![0.5; 512];
        let mut previous_peak = 0.0;
        for n in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let hz = 500.0 * 40_f64.powf(n);
            let out = render(
                &input,
                &Params {
                    brightness_hz: hz,
                    ..Params::default()
                },
            )
            .0;
            let peak = out.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
            assert!((peak - previous_peak).abs() < 0.2);
            previous_peak = peak;
        }
    }

    #[test]
    fn eq_at_extremes_is_finite() {
        let input = vec![0.4; 512];
        for (bass, treble) in [(-12.0, -12.0), (12.0, 12.0), (0.0, 12.0)] {
            let (l, r) = render(
                &input,
                &Params {
                    bass_db: bass,
                    treble_db: treble,
                    ..Params::default()
                },
            );
            assert!(l.iter().chain(r.iter()).all(|x| x.is_finite()));
        }
    }

    #[test]
    fn chorus_creates_stereo_difference() {
        // 4096 frames ≈ 85 ms: supera los 22 ms del delay base y deja que el
        // LFO desfasado llene ambas líneas antes de comparar canales.
        let input: Vec<f64> = (0..4096)
            .map(|i| (std::f64::consts::TAU * 220.0 * i as f64 / 48_000.0).sin())
            .collect();
        let (l, r) = render(
            &input,
            &Params {
                chorus_depth: 0.8,
                ..Params::default()
            },
        );
        let difference: f64 = l.iter().zip(r.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(difference > 0.0);
    }

    #[test]
    fn tremolo_modulates_amplitude() {
        let input = vec![0.5; 2048];
        let (l, _) = render(
            &input,
            &Params {
                tremolo_rate_hz: 4.0,
                tremolo_depth: 1.0,
                ..Params::default()
            },
        );
        let peak = l.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        assert!(peak > 0.0);
        assert!(l.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn reverb_is_deterministic_and_finite() {
        let input = impulse(2048);
        let params = Params {
            reverb: 0.7,
            ..Params::default()
        };
        let a = render(&input, &params);
        let b = render(&input, &params);
        assert_eq!(a.0, b.0);
        assert_eq!(a.1, b.1);
        assert!(a.0.iter().chain(a.1.iter()).all(|x| x.is_finite()));
    }

    #[test]
    fn reverb_wet_zero_matches_wet_off() {
        let input = impulse(1024);
        let on = render(&input, &Params::default()).0;
        let off = render(
            &input,
            &Params {
                reverb: 0.0,
                ..Params::default()
            },
        )
        .0;
        assert_eq!(on, off);
    }

    #[test]
    fn limit_keeps_output_bounded() {
        let input = vec![4.0; 1024];
        let (l, r) = render(
            &input,
            &Params {
                volume: 1.0,
                drive: 1.0,
                ..Params::default()
            },
        );
        assert!(l.iter().chain(r.iter()).all(|x| x.abs() <= 1.0));
    }

    #[test]
    fn reset_returns_to_initial_state() {
        let mut bus = bus();
        let params = Params {
            reverb: 1.0,
            chorus_depth: 1.0,
            ..Params::default()
        };
        let mut l = vec![0.5; 512];
        let mut r = l.clone();
        bus.process(&mut l, &mut r, &params, 1.0);
        bus.reset();
        let mut a = vec![0.0; 512];
        let mut b = a.clone();
        bus.process(&mut a, &mut b, &Params::default(), 0.0);
        assert!(a.iter().all(|x| x.is_finite()));
    }
}
