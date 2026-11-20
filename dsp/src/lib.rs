//! Núcleo DSP puro de Khodes.
//!
//! Este crate no conoce TRUCE, Slint, el host ni el sistema de archivos.
//! Recibe eventos y parámetros ya traducidos y produce audio.
//!
//! Ver `docs/ARCHITECTURE.md`, `docs/VOICE_SPEC.md` y `docs/MIDI.md`.

#![forbid(unsafe_code)]

pub mod allocator;
pub mod params;
pub mod voice;

pub use allocator::{Allocator, MAX_VOICES, SlotState};
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
    NoteOn {
        channel: u8,
        note: u8,
        velocity: f64,
        /// ID interno asignado por el wrapper; `-1` si el host no lo expone.
        note_id: i32,
    },
    NoteOff {
        channel: u8,
        note: u8,
        note_id: i32,
    },
    PitchBend {
        semitones: f64,
    },
    Sustain {
        channel: u8,
        enabled: bool,
    },
    ModWheel {
        amount: f64,
    },
    /// CC123: libera las voces con release normal.
    AllNotesOff,
    /// CC120: corta las voces con fade corto.
    AllSoundOff,
    /// CC121: centra bend, desactiva sustain y limpia modulación.
    ResetControllers,
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

/// Motor polifónico. Toda la memoria se reserva en `prepare`.
pub struct Engine {
    sample_rate: f64,
    noise: Vec<f64>,
    mix: Vec<f64>,
    scratch: Vec<f64>,
    freq_buf: Vec<f64>,
    allocator: Allocator,
    params: Params,
    bend_semitones: f64,
    mod_wheel: f64,
    freq_mult: f64,
    smooth_coeff: f64,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            noise: Vec::new(),
            mix: Vec::new(),
            scratch: Vec::new(),
            freq_buf: Vec::new(),
            allocator: Allocator::default(),
            params: Params::default(),
            bend_semitones: 0.0,
            mod_wheel: 0.0,
            freq_mult: 1.0,
            smooth_coeff: 0.0,
        }
    }
}

impl Engine {
    /// Reserva buffers y coeficientes. Puede asignar; no se llama en `process`.
    pub fn prepare(&mut self, sample_rate: f64, max_block_size: usize) {
        let sr = sample_rate.max(1.0);
        self.sample_rate = sr;
        let len = (voice::CLICK_SECONDS * sr).ceil() as usize;
        self.noise.resize(len, 0.0);
        let mut seed = 0x243F_6A88_85A3_08D3_u64;
        for sample in &mut self.noise {
            *sample = next_random(&mut seed);
        }
        self.mix.resize(max_block_size.max(1), 0.0);
        self.scratch.resize(max_block_size.max(1), 0.0);
        self.freq_buf.resize(max_block_size.max(1), 1.0);
        self.smooth_coeff = 1.0 - (-1.0 / (SMOOTHING_SECONDS * sr)).exp();
        self.allocator.prepare(sr);
        self.reset();
    }

    /// Reinicia todo el estado transitorio sin liberar memoria.
    pub fn reset(&mut self) {
        self.allocator.reset();
        self.bend_semitones = 0.0;
        self.mod_wheel = 0.0;
        self.freq_mult = 1.0;
    }

    #[cfg(test)]
    pub fn active_voices(&self) -> usize {
        self.allocator.active_count()
    }

    #[cfg(test)]
    pub fn mod_wheel(&self) -> f64 {
        self.mod_wheel
    }

    fn apply_event(&mut self, body: &EventBody) {
        match *body {
            EventBody::NoteOn {
                channel,
                note,
                velocity,
                note_id,
            } => {
                self.allocator.note_on(
                    channel,
                    note,
                    velocity,
                    note_id,
                    &self.params,
                    self.sample_rate,
                );
            }
            EventBody::NoteOff {
                channel,
                note,
                note_id,
            } => self.allocator.note_off(channel, note, note_id),
            EventBody::PitchBend { semitones } => {
                let range = f64::from(self.params.bend_range);
                self.bend_semitones = semitones.clamp(-range, range);
            }
            EventBody::Sustain { channel, enabled } => {
                self.allocator.set_sustain(channel, enabled);
            }
            EventBody::ModWheel { amount } => {
                self.mod_wheel = amount.clamp(0.0, 1.0);
            }
            EventBody::AllNotesOff => self.allocator.all_notes_off(),
            EventBody::AllSoundOff => self.allocator.all_sound_off(),
            EventBody::ResetControllers => {
                self.allocator.reset_controllers();
                self.bend_semitones = 0.0;
                self.mod_wheel = 0.0;
            }
        }
    }

    /// Procesa un bloque. No asigna memoria ni redimensiona buffers.
    pub fn process(&mut self, output: &mut [f64], events: &[Event], params: &Params) {
        self.params = *params;
        self.allocator.set_polyphony(params.polyphony);
        let frames = output.len().min(self.mix.len()).min(self.freq_buf.len());
        if frames == 0 {
            return;
        }
        let target_mult = 2.0_f64.powf(self.bend_semitones / 12.0);
        for mult in self.freq_buf[..frames].iter_mut() {
            self.freq_mult += self.smooth_coeff * (target_mult - self.freq_mult);
            *mult = self.freq_mult;
        }

        let mut pos = 0;
        while pos < frames {
            for event in events {
                if event.sample_offset == pos {
                    self.apply_event(&event.body);
                }
            }

            let mut end = frames;
            for event in events {
                let offset = event.sample_offset;
                if offset > pos && offset < end {
                    end = offset;
                }
            }

            self.allocator.render_segment(
                &mut self.mix[pos..end],
                &self.freq_buf[pos..end],
                &mut self.scratch[..end],
                &self.params,
                self.sample_rate,
                &self.noise,
            );
            pos = end;
        }

        output[..frames].copy_from_slice(&self.mix[..frames]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_note(note: u8, params: &Params, frames: usize) -> Vec<f64> {
        let mut engine = Engine::default();
        engine.prepare(48_000.0, frames);
        let mut out = vec![0.0; frames];
        engine.process(
            &mut out,
            &[Event::new(
                0,
                EventBody::NoteOn {
                    channel: 0,
                    note,
                    velocity: 1.0,
                    note_id: -1,
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
                    channel: 0,
                    note: 60,
                    velocity: 1.0,
                    note_id: -1,
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
                    channel: 0,
                    note: 60,
                    velocity: 1.0,
                    note_id: -1,
                },
            )],
            &params,
        );
        assert_eq!(engine.active_voices(), 1);
        engine.process(
            &mut out,
            &[Event::new(
                0,
                EventBody::NoteOff {
                    channel: 0,
                    note: 60,
                    note_id: -1,
                },
            )],
            &params,
        );
        engine.reset();
        assert_eq!(engine.active_voices(), 0);
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

    #[test]
    fn engine_plays_chords_and_honours_polyphony() {
        let mut engine = Engine::default();
        engine.prepare(48_000.0, 1024);
        let params = Params {
            polyphony: 8,
            ..Params::default()
        };
        let events: Vec<Event> = (60..76)
            .map(|note| {
                Event::new(
                    0,
                    EventBody::NoteOn {
                        channel: 0,
                        note,
                        velocity: 1.0,
                        note_id: i32::from(note),
                    },
                )
            })
            .collect();
        let mut out = [0.0; 1024];
        engine.process(&mut out, &events, &params);
        assert_eq!(engine.active_voices(), 8);
        assert!(out.iter().all(|x| x.is_finite()));
        assert!(out.iter().any(|x| *x != 0.0));
    }
}
