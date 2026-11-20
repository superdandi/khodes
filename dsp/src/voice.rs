//! Voz monofónica del piano eléctrico: dos senos, envelopes y key click.
//!
//! Réplica del comportamiento documentado en `docs/VOICE_SPEC.md` §3, con
//! fases preasignadas y sin asignaciones durante el render.
//!
//! Ver `docs/adr/0002-port-keys-rhodes-voice.md`.

use crate::params::Params;

/// Pequeño nivel de silencio usado por las rampas exponenciales de Web Audio.
const EXP_FLOOR: f64 = 0.001;
/// Nivel mínimo del decay para no alimentar una rampa exponencial con cero.
const DECAY_FLOOR: f64 = 0.0001;
/// Duración del buffer de ruido del key click.
pub const CLICK_SECONDS: f64 = 0.05;
/// Cola añadida por Keys al release (`rhodes.ts`: `end = ... + releaseT + 0.05`).
const RELEASE_TAIL_S: f64 = 0.05;
/// Frecuencia del highpass del key click.
const CLICK_HP_HZ: f64 = 3_000.0;

/// Convierte una nota MIDI a frecuencia con A4 = 440 Hz.
#[must_use]
pub fn midi_to_hz(note: u8) -> f64 {
    440.0 * 2.0_f64.powf((f64::from(note) - 69.0) / 12.0)
}

/// Fuerza a cero los subnormales para evitar coste de microarquitectura.
#[inline]
#[must_use]
pub fn flush_denormal(value: f64) -> f64 {
    if value.abs() < 1.0e-20 { 0.0 } else { value }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Curve {
    Linear,
    Exponential,
}

/// Envelope de etapas acotadas. Las rampas exponenciales se implementan como
/// un factor multiplicativo precalculado, sin `powf` por muestra.
#[derive(Clone, Copy, Debug)]
struct Env {
    stage: Stage,
    level: f64,
    start: f64,
    target: f64,
    slope: f64,
    step: f64,
    factor: f64,
    elapsed: usize,
    len: usize,
    curve: Curve,
    after_decay: Stage,
    decay_target: f64,
    decay_len: usize,
    sustain_level: f64,
    release_len: usize,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            stage: Stage::Idle,
            level: 0.0,
            start: 0.0,
            target: 0.0,
            slope: 0.0,
            step: 1.0,
            factor: 1.0,
            elapsed: 0,
            len: 1,
            curve: Curve::Linear,
            after_decay: Stage::Done,
            decay_target: EXP_FLOOR,
            decay_len: 1,
            sustain_level: 0.0,
            release_len: 1,
        }
    }
}

impl Env {
    fn begin(&mut self, stage: Stage, start: f64, target: f64, len: usize, curve: Curve) {
        self.stage = stage;
        self.level = start;
        self.start = start;
        self.target = target;
        self.len = len.max(1);
        self.elapsed = 0;
        self.curve = curve;
        self.slope = (target - start) / self.len as f64;
        self.step = if curve == Curve::Exponential && start > 0.0 && target > 0.0 {
            (target / start).powf(1.0 / self.len as f64)
        } else {
            1.0
        };
        self.factor = 1.0;
    }

    /// Envelope principal: ataque lineal, decay exponencial al sustain y
    /// release exponencial desde el nivel actual.
    fn trigger_adsr(
        &mut self,
        peak: f64,
        sustain: f64,
        attack_len: usize,
        decay_len: usize,
        release_len: usize,
    ) {
        self.sustain_level = sustain.max(DECAY_FLOOR);
        self.decay_target = self.sustain_level;
        self.decay_len = decay_len;
        self.release_len = release_len;
        self.after_decay = Stage::Sustain;
        self.begin(Stage::Attack, 0.0, peak, attack_len, Curve::Linear);
    }

    /// Envelope de campana o click: ataque lineal y caída exponencial, sin
    /// sostenimiento.
    fn trigger_one_shot(&mut self, peak: f64, attack_len: usize, decay_len: usize) {
        self.decay_target = EXP_FLOOR;
        self.decay_len = decay_len;
        self.after_decay = Stage::Done;
        self.begin(Stage::Attack, 0.0, peak, attack_len, Curve::Linear);
    }

    fn release(&mut self) {
        if matches!(self.stage, Stage::Idle | Stage::Done | Stage::Release) {
            return;
        }
        let from = self.level.max(1.0e-9);
        let to = EXP_FLOOR.min(from);
        self.begin(
            Stage::Release,
            from,
            to,
            self.release_len,
            Curve::Exponential,
        );
    }

    #[inline]
    fn tick(&mut self) -> f64 {
        match self.stage {
            Stage::Idle | Stage::Done => return 0.0,
            Stage::Sustain => return self.sustain_level,
            _ => {}
        }

        if self.curve == Curve::Linear {
            self.level = self.start + self.slope * self.elapsed as f64;
        } else {
            self.factor *= self.step;
            self.level = self.start * self.factor;
        }
        self.elapsed += 1;

        if self.elapsed >= self.len {
            self.level = self.target;
            match self.stage {
                Stage::Attack => {
                    let peak = self.target;
                    self.begin(
                        Stage::Decay,
                        peak.max(1.0e-9),
                        self.decay_target,
                        self.decay_len,
                        Curve::Exponential,
                    );
                }
                Stage::Decay => {
                    if self.after_decay == Stage::Sustain {
                        self.stage = Stage::Sustain;
                        self.level = self.sustain_level;
                    } else {
                        self.stage = Stage::Done;
                        self.level = 0.0;
                    }
                }
                Stage::Release => {
                    self.stage = Stage::Done;
                    self.level = 0.0;
                }
                _ => {}
            }
        }
        self.level
    }

    #[inline]
    const fn is_done(&self) -> bool {
        matches!(self.stage, Stage::Idle | Stage::Done)
    }
}

/// Estado de una voz. Toda la memoria se reserva antes del primer render.
#[derive(Clone, Debug)]
pub struct Voice {
    active: bool,
    note: u8,
    note_hz: f64,
    phase: f64,
    bell_phase: f64,
    vel_gain: f64,
    amp: Env,
    bell: Env,
    click: Env,
    click_pos: usize,
    click_len: usize,
    click_hp_in: f64,
    click_hp_out: f64,
    click_hp_a: f64,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            note: 0,
            note_hz: 440.0,
            phase: 0.0,
            bell_phase: 0.0,
            vel_gain: 1.0,
            amp: Env::default(),
            bell: Env::default(),
            click: Env::default(),
            click_pos: 0,
            click_len: 0,
            click_hp_in: 0.0,
            click_hp_out: 0.0,
            click_hp_a: 0.0,
        }
    }
}

impl Voice {
    /// Ganancia de velocity del contrato: `0.3 + 0.7 * (1 - s + s * v)`.
    #[must_use]
    pub fn velocity_gain(sensitivity: f64, velocity: f64) -> f64 {
        let s = sensitivity.clamp(0.0, 1.0);
        let v = velocity.clamp(0.0, 1.0);
        0.3 + 0.7 * (1.0 - s + s * v)
    }

    fn samples(seconds: f64, sample_rate: f64) -> usize {
        (seconds * sample_rate).round().max(1.0) as usize
    }

    /// Inicia una nota. No asigna memoria.
    pub fn note_on(&mut self, note: u8, velocity: f64, params: &Params, sample_rate: f64) {
        let sr = sample_rate.max(1.0);
        self.vel_gain = Self::velocity_gain(params.velocity_sensitivity, velocity);
        let peak = params.volume * self.vel_gain;
        self.note = note;
        self.note_hz = midi_to_hz(note);
        self.phase = 0.0;
        self.bell_phase = 0.0;
        self.active = true;

        let attack = Self::samples(params.attack_s, sr);
        let decay = Self::samples(params.decay_s, sr);
        let release = Self::samples(params.release_s + RELEASE_TAIL_S, sr);
        let sustain = (peak * params.sustain).max(DECAY_FLOOR);
        self.amp.trigger_adsr(peak, sustain, attack, decay, release);

        let bell_attack = Self::samples(params.attack_s * 0.6, sr);
        let bell_decay = Self::samples(params.decay_s * 0.6 + 0.02, sr);
        self.bell
            .trigger_one_shot(peak * 0.35, bell_attack, bell_decay);

        let click_attack = Self::samples(0.001, sr);
        let click_decay = Self::samples(0.03, sr);
        self.click
            .trigger_one_shot(peak * params.key_click, click_attack, click_decay);
        self.click_len = (CLICK_SECONDS * sr).ceil() as usize;
        self.click_pos = 0;
        self.click_hp_in = 0.0;
        self.click_hp_out = 0.0;
        let dt = 1.0 / sr;
        let rc = 1.0 / (std::f64::consts::TAU * CLICK_HP_HZ);
        self.click_hp_a = rc / (rc + dt);
    }

    /// Libera la nota si coincide con la voz activa. La campana sigue su ciclo.
    pub fn note_off(&mut self, note: u8) {
        if self.active && self.note == note {
            self.amp.release();
        }
    }

    /// Fuerza el fin de la voz con un fade corto (robo / all sound off).
    pub fn kill(&mut self) {
        self.amp.release();
    }

    pub const fn is_active(&self) -> bool {
        self.active
    }

    pub const fn note(&self) -> u8 {
        self.note
    }

    #[cfg(test)]
    pub const fn envelope_level(&self) -> f64 {
        self.amp.level
    }

    /// Aísla el oscilador fundamental para medir su frecuencia sin el parcial
    /// de campana ni el click. Solo existe en tests.
    #[cfg(test)]
    pub fn render_fundamental_only(&mut self, out: &mut [f64], sample_rate: f64) {
        let inc = self.note_hz / sample_rate.max(1.0);
        for sample in out.iter_mut() {
            *sample = (std::f64::consts::TAU * self.phase).sin();
            self.phase = (self.phase + inc).rem_euclid(1.0);
        }
    }

    /// Renderiza `out` con la frecuencia compartida `freq_mult`.
    ///
    /// `freq_mult` se suaviza hacia `target_mult` con `smooth_coeff` para que el
    /// pitch bend y el fine tune no produzcan saltos.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        out: &mut [f64],
        freq_mult: &mut f64,
        target_mult: f64,
        smooth_coeff: f64,
        params: &Params,
        sample_rate: f64,
        noise: &[f64],
    ) {
        let sr = sample_rate.max(1.0);
        let fundamental_hz = self.note_hz * 2.0_f64.powf(params.fine_tune_cents / 1200.0);
        let bell_hz = fundamental_hz * 2.0;
        let bell_atten = if bell_hz >= 0.45 * sr {
            ((0.5 * sr - bell_hz) / (0.05 * sr)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let phase_inc = fundamental_hz / sr;
        let bell_phase_inc = bell_hz / sr;

        for sample in out.iter_mut() {
            *freq_mult += smooth_coeff * (target_mult - *freq_mult);

            let g1 = self.amp.tick();
            let g2 = self.bell.tick();

            let fundamental = (std::f64::consts::TAU * self.phase).sin();
            self.phase = (self.phase + phase_inc * *freq_mult).rem_euclid(1.0);

            let bell = (std::f64::consts::TAU * self.bell_phase).sin() * bell_atten;
            self.bell_phase = (self.bell_phase + bell_phase_inc * *freq_mult).rem_euclid(1.0);

            let mut click = 0.0;
            if self.click_pos < self.click_len && !noise.is_empty() {
                let raw = noise[self.click_pos % noise.len()];
                self.click_hp_out = self.click_hp_a * (self.click_hp_out + raw - self.click_hp_in);
                self.click_hp_in = raw;
                click = self.click_hp_out * self.click.tick();
                self.click_pos += 1;
            }

            *sample = flush_denormal(fundamental * g1 + bell * g2 + click);
        }

        if self.amp.is_done() && self.bell.is_done() && self.click_pos >= self.click_len {
            self.active = false;
        }
    }

    /// Limpia por completo el estado de la voz.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_frequency_contract() {
        assert!((midi_to_hz(69) - 440.0).abs() < 1e-9);
        assert!((midi_to_hz(60) - 261.6255653005986).abs() < 1e-9);
        assert!((midi_to_hz(21) - 27.5).abs() < 1e-9);
        assert!((midi_to_hz(108) - 4186.009044809578).abs() < 1e-8);
    }

    #[test]
    fn exponential_stages_reach_their_targets() {
        let mut env = Env::default();
        env.trigger_adsr(0.3, 0.18, 10, 20, 30);
        for _ in 0..10 {
            let _ = env.tick();
        }
        assert_eq!(env.stage, Stage::Decay);
        for _ in 0..20 {
            let _ = env.tick();
        }
        assert_eq!(env.stage, Stage::Sustain);
        assert!((env.level - 0.18).abs() < 1e-12);
        env.release();
        for _ in 0..29 {
            let _ = env.tick();
        }
        assert_eq!(env.stage, Stage::Release);
        let _ = env.tick();
        assert_eq!(env.stage, Stage::Done);
        assert_eq!(env.level, 0.0);
    }

    #[test]
    fn oscillator_frequency_is_within_one_cent_for_four_notes() {
        let sr = 48_000.0;
        let params = Params::default();
        for note in [21, 60, 69, 108] {
            let mut voice = Voice::default();
            voice.note_on(note, 1.0, &params, sr);
            let mut buf = vec![0.0; sr as usize];
            voice.render_fundamental_only(&mut buf, sr);
            let mut crossings = Vec::new();
            for i in 1..buf.len() {
                if buf[i - 1] < 0.0 && buf[i] >= 0.0 {
                    let fraction = -buf[i - 1] / (buf[i] - buf[i - 1]);
                    crossings.push(i as f64 - 1.0 + fraction);
                }
            }
            assert!(crossings.len() >= 2, "not enough cycles for note {note}");
            let cycles = (crossings.len() - 1) as f64;
            let duration = (crossings[crossings.len() - 1] - crossings[0]) / sr;
            let measured = cycles / duration;
            let expected = midi_to_hz(note);
            let cents = 1200.0 * (measured / expected).log2();
            assert!(
                cents.abs() <= 1.0,
                "note {note}: {measured} Hz vs {expected} Hz = {cents} cents"
            );
        }
    }

    #[test]
    fn envelope_stage_lengths_follow_the_contract() {
        let sr = 48_000.0;
        let params = Params {
            attack_s: 0.01,
            decay_s: 0.05,
            release_s: 0.04,
            sustain: 0.5,
            ..Params::default()
        };
        let mut voice = Voice::default();
        voice.note_on(60, 1.0, &params, sr);
        let mut scratch = [0.0; 1];
        for _ in 0..479 {
            voice.render(&mut scratch, &mut 1.0, 1.0, 0.0, &params, sr, &[]);
        }
        assert!((voice.envelope_level() - params.volume).abs() < 0.002);
        for _ in 0..2400 {
            voice.render(&mut scratch, &mut 1.0, 1.0, 0.0, &params, sr, &[]);
        }
        assert!((voice.envelope_level() - params.volume * 0.5).abs() < 0.002);
        voice.note_off(60);
        for _ in 0..4320 {
            voice.render(&mut scratch, &mut 1.0, 1.0, 0.0, &params, sr, &[]);
        }
        assert_eq!(voice.envelope_level(), 0.0);
    }

    #[test]
    fn velocity_floor_matches_the_contract() {
        assert!((Voice::velocity_gain(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((Voice::velocity_gain(1.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((Voice::velocity_gain(1.0, 0.0) - 0.3).abs() < 1e-12);
        assert!((Voice::velocity_gain(0.5, 1.0) - 1.0).abs() < 1e-12);
    }
}
