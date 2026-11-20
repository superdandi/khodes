//! Traducción de eventos MIDI de TRUCE al vocabulario del DSP.
//!
//! TRUCE no expone el `note_id` de CLAP en `EventBody`, así que el wrapper
//! asigna sus propios IDs secuenciales y los recuerda por (canal, nota). Así un
//! note-off tardío no puede liberar una voz que ya fue robada.
//!
//! Ver `docs/MIDI.md` §8 y §9 y `docs/PLAN.md` M3-06.

use khodes_dsp::{Event as DspEvent, EventBody as DspEventBody};
use truce::prelude64::*;

/// Máximo de eventos traducidos por bloque. Array en stack, sin asignaciones.
pub const MAX_EVENTS_PER_BLOCK: usize = 256;
/// 16 canales × 128 notas.
const ACTIVE_IDS_LEN: usize = 16 * 128;

/// Asigna IDs internos y traduce la lista de eventos del host al DSP.
#[derive(Clone, Debug)]
pub struct MidiMapper {
    active_ids: [i32; ACTIVE_IDS_LEN],
    next_id: i32,
}

impl Default for MidiMapper {
    fn default() -> Self {
        Self {
            active_ids: [-1; ACTIVE_IDS_LEN],
            next_id: 1,
        }
    }
}

impl MidiMapper {
    /// Limpia el mapa al reiniciar el plugin. No libera memoria.
    pub fn reset(&mut self) {
        self.active_ids = [-1; ACTIVE_IDS_LEN];
        self.next_id = 1;
    }

    fn slot(channel: u8, note: u8) -> usize {
        usize::from(channel & 15) * 128 + usize::from(note & 127)
    }

    fn allocate_id(&mut self, channel: u8, note: u8) -> i32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id <= 0 {
            self.next_id = 1;
        }
        self.active_ids[Self::slot(channel, note)] = id;
        id
    }

    fn consume_id(&mut self, channel: u8, note: u8) -> i32 {
        let slot = Self::slot(channel, note);
        let id = self.active_ids[slot];
        self.active_ids[slot] = -1;
        id
    }

    /// Traduce los eventos del bloque. Devuelve cuántos se copiaron en `out`.
    pub fn translate(
        &mut self,
        events: &EventList,
        out: &mut [DspEvent; MAX_EVENTS_PER_BLOCK],
        frames: usize,
        bend_range: u8,
    ) -> usize {
        let mut count = 0;
        for event in events.iter() {
            let offset = event.sample_offset as usize;
            if offset >= frames || count == out.len() {
                continue;
            }
            let body = match event.body {
                EventBody::NoteOn {
                    channel,
                    note,
                    velocity,
                    ..
                } => {
                    let note_id = self.allocate_id(channel, note);
                    Some(DspEventBody::NoteOn {
                        channel,
                        note,
                        velocity: f64::from(velocity) / 127.0,
                        note_id,
                    })
                }
                EventBody::NoteOff { channel, note, .. } => {
                    let note_id = self.consume_id(channel, note);
                    Some(DspEventBody::NoteOff {
                        channel,
                        note,
                        note_id,
                    })
                }
                EventBody::NoteOn2 {
                    channel,
                    note,
                    velocity,
                    ..
                } => {
                    let note_id = self.allocate_id(channel, note);
                    Some(DspEventBody::NoteOn {
                        channel,
                        note,
                        velocity: f64::from(velocity) / 65_535.0,
                        note_id,
                    })
                }
                EventBody::NoteOff2 { channel, note, .. } => {
                    let note_id = self.consume_id(channel, note);
                    Some(DspEventBody::NoteOff {
                        channel,
                        note,
                        note_id,
                    })
                }
                EventBody::PitchBend { value, .. } => {
                    let normalized = (f64::from(value) - 8192.0) / 8192.0;
                    Some(DspEventBody::PitchBend {
                        semitones: normalized * f64::from(bend_range),
                    })
                }
                EventBody::ControlChange {
                    channel, cc, value, ..
                } => match cc {
                    1 => Some(DspEventBody::ModWheel {
                        amount: f64::from(value) / 127.0,
                    }),
                    64 => Some(DspEventBody::Sustain {
                        channel,
                        enabled: value >= 64,
                    }),
                    120 => Some(DspEventBody::AllSoundOff),
                    121 => Some(DspEventBody::ResetControllers),
                    123 => Some(DspEventBody::AllNotesOff),
                    _ => None,
                },
                _ => None,
            };
            if let Some(body) = body {
                out[count] = DspEvent::new(offset, body);
                count += 1;
            }
        }
        count
    }
}
