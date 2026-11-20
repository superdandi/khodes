//! Núcleo DSP puro de Khodes.
//!
//! Este crate no conoce TRUCE, Slint, el host ni el sistema de archivos.
//! Recibe eventos y parámetros ya traducidos y produce audio.
//!
//! Ver `docs/ARCHITECTURE.md` y `docs/VOICE_SPEC.md`.

#![forbid(unsafe_code)]

pub mod params;
pub mod voice;

pub use params::Params;
pub use voice::{Voice, flush_denormal, midi_to_hz};

/// Versión del contrato de sonido.
///
/// Cambia solo cuando el audio cambia de forma intencional. Es independiente
/// de la versión del producto y de la versión del códec de presets.
pub const DSP_CONTRACT_VERSION: u32 = 1;

/// Constante de suavizado de parámetros: ~20 ms (`VOICE_SPEC` §6.1).
const SMOOTHING_SECONDS: f64 = 0.02;
/// Paso de incremento del PRNG (SplitMix64).
const PRNG_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;

/// Evento ya traducido por el wrapper, con offset de muestra dentro del bloque.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub sample_offset: usize,
    pub body: EventBody,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventBody {
    NoteOn { note: u8, velocity: f64 },
    NoteOff { note: u8 },
    PitchBend { semitones: f64 },
    Sustain { enabled: bool },
    ModWheel { amount: f64 },
    AllNotesOff,
}

impl Event {
    #[must_use]
    pub const fn new(sample_offset: usize, body: EventBody) -> Self {
        Self {
            sample_offset,
            body,
        }
    }
}

/// PRNG determinista para el ruido del key click y la IR de reverb.
#[must_use]
pub fn next_random(state: &mut u64) -> f64 {
    *state = state.wrapping_add(PRNG_INCREMENT);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let bits = (z >> 11) as f64 / (1_u64 << 53) as f64;
    bits * 2.0 - 1.0
}

/// Motor monofónico de M2. En M3 se sustituye la voz por un allocator.
pub struct Engine {
    sample_rate: f64,
    noise: Vec<f64>,
    voice: Voice,
    params: Params,
    bend_semitones: f64,
    mod_wheel: f64,
    sustain: bool,
    sustain_pending: bool,
    freq_mult: f64,
    smooth_coeff: f64,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            noise: Vec::new(),
            voice: Voice::default(),
            params: Params::default(),
            bend_semitones: 0.0,
            mod_wheel: 0.0,
            sustain: false,
            sustain_pending: false,
            freq_mult: 1.0,
            smooth_coeff: 0.0,
        }
    }
}

impl Engine {
    /// Reserva el ruido del click y calcula coeficientes. Puede asignar.
    pub fn prepare(&mut self, sample_rate: f64, max_block_size: usize) {
        let _ = max_block_size;
        let sr = sample_rate.max(1.0);
        self.sample_rate = sr;
        let len = (voice::CLICK_SECONDS * sr).ceil() as usize;
        self.noise.resize(len, 0.0);
        let mut seed = 0x243F_6A88_85A3_08D3_u64;
        for sample in &mut self.noise {
            *sample = next_random(&mut seed);
        }
        self.smooth_coeff = 1.0 - (-1.0 / (SMOOTHING_SECONDS * sr)).exp();
        self.reset();
    }

    /// Reinicia todo el estado transitorio sin liberar memoria.
    pub fn reset(&mut self) {
        self.voice.reset();
        self.bend_semitones = 0.0;
        self.mod_wheel = 0.0;
        self.sustain = false;
        self.sustain_pending = false;
        self.freq_mult = 1.0;
    }

    #[cfg(test)]
    pub const fn voice(&self) -> &Voice {
        &self.voice
    }

    #[cfg(test)]
    pub fn mod_wheel(&self) -> f64 {
        self.mod_wheel
    }

    fn apply_event(&mut self, body: &EventBody) {
        match *body {
            EventBody::NoteOn { note, velocity } => {
                self.sustain_pending = false;
                self.voice
                    .note_on(note, velocity, &self.params, self.sample_rate);
            }
            EventBody::NoteOff { note } => {
                if self.sustain {
                    self.sustain_pending = true;
                } else {
                    self.voice.note_off(note);
                }
            }
            EventBody::PitchBend { semitones } => {
                self.bend_semitones = semitones.clamp(
                    -(f64::from(self.params.bend_range)),
                    f64::from(self.params.bend_range),
                );
            }
            EventBody::Sustain { enabled } => {
                self.sustain = enabled;
                if !enabled && self.sustain_pending {
                    self.sustain_pending = false;
                    let note = self.voice.note();
                    self.voice.note_off(note);
                }
            }
            EventBody::ModWheel { amount } => {
                self.mod_wheel = amount.clamp(0.0, 1.0);
            }
            EventBody::AllNotesOff => {
                self.sustain_pending = false;
                self.voice.kill();
            }
        }
    }

    /// Procesa un bloque. No asigna memoria ni redimensiona buffers.
    pub fn process(&mut self, output: &mut [f64], events: &[Event], params: &Params) {
        self.params = *params;
        let target_mult = 2.0_f64.powf(self.bend_semitones / 12.0);
        let sr = self.sample_rate;
        let len = output.len();

        let mut pos = 0;
        while pos < len {
            for event in events {
                if event.sample_offset == pos {
                    self.apply_event(&event.body);
                }
            }

            let mut end = len;
            for event in events {
                let offset = event.sample_offset;
                if offset > pos && offset < end {
                    end = offset;
                }
            }

            self.voice.render(
                &mut output[pos..end],
                &mut self.freq_mult,
                target_mult,
                self.smooth_coeff,
                &self.params,
                sr,
                &self.noise,
            );
            pos = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stereo_block() -> [f64; 1024] {
        [0.0; 1024]
    }

    fn render_note(note: u8, params: &Params, frames: usize) -> Vec<f64> {
        let mut engine = Engine::default();
        engine.prepare(48_000.0, frames);
        let mut out = vec![0.0; frames];
        engine.process(
            &mut out,
            &[Event::new(
                0,
                EventBody::NoteOn {
                    note,
                    velocity: 1.0,
                },
            )],
            params,
        );
        out
    }

    #[test]
    fn prng_is_deterministic_and_bounded() {
        let mut a = 1_u64;
        let mut b = 1_u64;
        for _ in 0..1000 {
            let x = next_random(&mut a);
            assert_eq!(x, next_random(&mut b));
            assert!((-1.0..=1.0).contains(&x));
        }
    }

    #[test]
    fn output_is_finite_for_all_default_parameters() {
        let out = render_note(60, &Params::default(), 1024);
        assert!(out.iter().all(|x| x.is_finite()));
        let _ = stereo_block();
    }

    #[test]
    fn note_on_produces_audio() {
        let params = Params {
            sustain: 1.0,
            ..Params::default()
        };
        let out = render_note(60, &params, 4096);
        let peak = out.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        assert!(peak > 0.05, "peak was {peak}");
    }

    #[test]
    fn clock_and_repeatability() {
        let params = Params {
            key_click: 1.0,
            ..Params::default()
        };
        let a = render_note(60, &params, 2048);
        let b = render_note(60, &params, 2048);
        assert_eq!(a, b);
        let clean = render_note(
            60,
            &Params {
                key_click: 0.0,
                ..Params::default()
            },
            2048,
        );
        let rms = |buf: &[f64]| (buf.iter().map(|x| x * x).sum::<f64>() / buf.len() as f64).sqrt();
        assert!(rms(&a[..48]) > rms(&clean[..48]));
    }

    #[test]
    fn events_respect_sample_offset() {
        let mut engine = Engine::default();
        engine.prepare(48_000.0, 512);
        let mut out = [0.0; 512];
        engine.process(
            &mut out,
            &[Event::new(
                256,
                EventBody::NoteOn {
                    note: 60,
                    velocity: 1.0,
                },
            )],
            &Params::default(),
        );
        assert!(out[..256].iter().all(|x| *x == 0.0));
        assert!(out[256..].iter().any(|x| *x != 0.0));
    }

    #[test]
    fn note_off_and_reset_stop_the_voice() {
        let mut engine = Engine::default();
        engine.prepare(48_000.0, 1024);
        let mut out = [0.0; 1024];
        let params = Params::default();
        engine.process(
            &mut out,
            &[Event::new(
                0,
                EventBody::NoteOn {
                    note: 60,
                    velocity: 1.0,
                },
            )],
            &params,
        );
        assert!(engine.voice().is_active());
        engine.process(
            &mut out,
            &[Event::new(0, EventBody::NoteOff { note: 60 })],
            &params,
        );
        engine.reset();
        assert!(!engine.voice().is_active());
    }

    #[test]
    fn pitch_bend_and_mod_wheel_are_bounded() {
        let mut engine = Engine::default();
        engine.prepare(48_000.0, 256);
        let mut out = [0.0; 256];
        let params = Params::default();
        engine.process(
            &mut out,
            &[
                Event::new(0, EventBody::PitchBend { semitones: 99.0 }),
                Event::new(0, EventBody::ModWheel { amount: 2.0 }),
            ],
            &params,
        );
        assert_eq!(engine.mod_wheel(), 1.0);
        assert!(out.iter().all(|x| x.is_finite()));
    }
}
