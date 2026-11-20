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
                    7 => Some(DspEventBody::MasterVolume {
                        level: f64::from(value) / 127.0,
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

        // Orden estable por offset (M7-07). Los wrappers ya suelen entregar la
        // lista ordenada, pero el contrato exige conservar el orden de llegada
        // de eventos con el mismo offset aunque un host no lo haga. Inserción
        // sobre el array de stack: sin asignaciones y acotada a 256 eventos.
        for i in 1..count {
            let mut j = i;
            while j > 0 && out[j - 1].sample_offset > out[j].sample_offset {
                out.swap(j - 1, j);
                j -= 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use khodes_dsp::{Engine, Params as DspParams};

    const FRAMES: usize = 256;

    fn host(entries: &[(u32, EventBody)]) -> EventList {
        let mut list = EventList::with_capacity(entries.len());
        for (offset, body) in entries {
            list.push(Event::new(*offset, *body));
        }
        list
    }

    fn note_on(channel: u8, note: u8) -> EventBody {
        EventBody::NoteOn {
            group: 0,
            channel,
            note,
            velocity: 100,
        }
    }

    fn note_off(channel: u8, note: u8) -> EventBody {
        EventBody::NoteOff {
            group: 0,
            channel,
            note,
            velocity: 0,
        }
    }

    fn cc(cc_number: u8, value: u8) -> EventBody {
        EventBody::ControlChange {
            group: 0,
            channel: 0,
            cc: cc_number,
            value,
        }
    }

    fn translated(events: &EventList) -> Vec<DspEvent> {
        let mut mapper = MidiMapper::default();
        let mut out = [DspEvent::new(0, DspEventBody::AllNotesOff); MAX_EVENTS_PER_BLOCK];
        let count = mapper.translate(events, &mut out, FRAMES, 2);
        out[..count].to_vec()
    }

    fn engine_with(events: &[DspEvent]) -> Engine {
        let mut engine = Engine::default();
        engine.prepare(48_000.0, FRAMES);
        let mut left = vec![0.0; FRAMES];
        let mut right = vec![0.0; FRAMES];
        engine.process_stereo(&mut left, &mut right, events, &DspParams::default());
        engine
    }

    fn note_id(body: DspEventBody) -> i32 {
        match body {
            DspEventBody::NoteOn { note_id, .. } | DspEventBody::NoteOff { note_id, .. } => note_id,
            _ => panic!("se esperaba un evento de nota"),
        }
    }

    fn semitones(body: DspEventBody) -> f64 {
        match body {
            DspEventBody::PitchBend { semitones } => semitones,
            _ => panic!("se esperaba un pitch bend"),
        }
    }

    /// M7-08: la misma nota en dos canales son dos voces con IDs distintos;
    /// el note-off de un canal no libera la voz del otro.
    #[test]
    fn two_channels_same_note_have_independent_holds() {
        let events = host(&[
            (0, note_on(0, 60)),
            (0, note_on(1, 60)),
            (128, note_off(0, 60)),
        ]);
        let out = translated(&events);
        assert_eq!(out.len(), 3);
        assert_ne!(note_id(out[0].body), note_id(out[1].body));
        assert_eq!(note_id(out[2].body), note_id(out[0].body));
        assert_eq!(engine_with(&out).active_voices(), 1);
    }

    /// M7-09: solapes con note-off cruzado solo liberan la asignación correcta.
    #[test]
    fn overlapping_holds_and_crossed_note_off_use_their_own_ids() {
        let events = host(&[
            (0, note_on(0, 60)),
            (16, note_on(0, 64)),
            (32, note_off(0, 60)),
            (48, note_on(0, 60)),
            (64, note_off(0, 64)),
        ]);
        let out = translated(&events);
        let ids: Vec<i32> = out.iter().map(|e| note_id(e.body)).collect();
        assert_eq!(ids, vec![1, 2, 1, 3, 2]);
        assert_eq!(engine_with(&out).active_voices(), 1);
    }

    /// M7-10: un note-off en el mismo bloque conserva su offset y se aplica.
    #[test]
    fn note_on_and_note_off_in_the_same_block_keep_offsets() {
        let events = host(&[(0, note_on(0, 60)), (10, note_off(0, 60))]);
        let out = translated(&events);
        assert_eq!(
            out.iter().map(|e| e.sample_offset).collect::<Vec<_>>(),
            vec![0, 10]
        );
        assert_eq!(engine_with(&out).active_voices(), 0);
    }

    /// M7-11: centro, extremos y retorno del bend de 14 bits.
    #[test]
    fn pitch_bend_covers_center_extremes_and_return() {
        let events = host(&[
            (
                0,
                EventBody::PitchBend {
                    group: 0,
                    channel: 0,
                    value: 0,
                },
            ),
            (
                8,
                EventBody::PitchBend {
                    group: 0,
                    channel: 0,
                    value: 8192,
                },
            ),
            (
                16,
                EventBody::PitchBend {
                    group: 0,
                    channel: 0,
                    value: 16383,
                },
            ),
            (
                24,
                EventBody::PitchBend {
                    group: 0,
                    channel: 0,
                    value: 8192,
                },
            ),
        ]);
        let out = translated(&events);
        let values: Vec<f64> = out.iter().map(|e| semitones(e.body)).collect();
        assert_eq!(values.len(), 4);
        assert!(
            (values[0] + 2.0).abs() < 1e-3,
            "extremo negativo: {}",
            values[0]
        );
        assert_eq!(values[1], 0.0, "centro exacto");
        assert!(
            (values[2] - 2.0).abs() < 1e-3,
            "extremo positivo: {}",
            values[2]
        );
        assert_eq!(values[3], 0.0, "retorno al centro");
    }

    /// M7-12: All Notes Off libera también notas retenidas por el pedal.
    #[test]
    fn all_notes_off_releases_sustained_notes() {
        let events = host(&[
            (0, cc(64, 127)),
            (0, note_on(0, 60)),
            (16, note_off(0, 60)),
            (32, cc(123, 0)),
        ]);
        let out = translated(&events);
        assert_eq!(engine_with(&out).active_voices(), 0);
        assert!(matches!(
            out[0].body,
            DspEventBody::Sustain { enabled: true, .. }
        ));
        assert!(matches!(out[3].body, DspEventBody::AllNotesOff));
    }

    /// M7-13: un host sin offsets aplica todo al inicio del bloque.
    #[test]
    fn host_without_offsets_applies_everything_at_block_start() {
        let events = host(&[(0, note_on(0, 60)), (0, cc(1, 64)), (0, cc(7, 64))]);
        let out = translated(&events);
        assert!(out.iter().all(|e| e.sample_offset == 0));
        let mut engine = Engine::default();
        engine.prepare(48_000.0, FRAMES);
        let mut left = vec![0.0; FRAMES];
        let mut right = vec![0.0; FRAMES];
        engine.process_stereo(&mut left, &mut right, &out, &DspParams::default());
        assert!(
            left.iter().any(|s| s.abs() > 0.0),
            "la nota suena desde el inicio"
        );
        assert!(left.iter().all(|s| s.is_finite()));
    }

    /// M7-05: CC7 viaja como ganancia de performance y CC121 la restaura.
    #[test]
    fn cc7_is_a_performance_gain_and_cc121_restores_it() {
        let events = host(&[(0, cc(7, 0))]);
        let out = translated(&events);
        match out[0].body {
            DspEventBody::MasterVolume { level } => assert_eq!(level, 0.0),
            other => panic!("se esperaba MasterVolume, se recibió {other:?}"),
        }
        assert_eq!(engine_with(&out).cc_volume(), 0.0);

        let events = host(&[(0, cc(7, 64)), (0, cc(121, 0))]);
        let out = translated(&events);
        match out[0].body {
            DspEventBody::MasterVolume { level } => assert!(level > 0.0),
            other => panic!("se esperaba MasterVolume, se recibió {other:?}"),
        }
        assert!(matches!(out[1].body, DspEventBody::ResetControllers));
        assert_eq!(engine_with(&out).cc_volume(), 1.0);
    }

    /// M7-07: el orden por offset es estable aunque el host desordene la lista.
    #[test]
    fn events_are_sorted_by_offset_preserving_arrival_order_on_ties() {
        let events = host(&[(40, note_on(0, 64)), (10, note_off(0, 60)), (10, cc(1, 64))]);
        let out = translated(&events);
        assert_eq!(
            out.iter().map(|e| e.sample_offset).collect::<Vec<_>>(),
            vec![10, 10, 40]
        );
        assert!(matches!(out[0].body, DspEventBody::NoteOff { .. }));
        assert!(matches!(out[1].body, DspEventBody::ModWheel { .. }));
        assert!(matches!(out[2].body, DspEventBody::NoteOn { .. }));
    }

    #[test]
    fn stale_and_out_of_block_events_are_ignored() {
        let events = host(&[(0, note_off(0, 60)), (FRAMES as u32, note_on(0, 60))]);
        let out = translated(&events);
        assert_eq!(out.len(), 1, "el evento fuera del bloque se descarta");
        assert_eq!(
            note_id(out[0].body),
            -1,
            "note-off huérfano usa ID wildcard"
        );
        assert_eq!(engine_with(&out).active_voices(), 0);
    }
}
