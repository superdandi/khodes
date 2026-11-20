//! Khodes — piano eléctrico de tine sintetizado.
//!
//! Este crate contiene la integración de producto: parámetros, MIDI, estado y
//! editor. El DSP vive en `khodes-dsp` y los presets en `khodes-kit`.
//!
//! Ver `docs/ARCHITECTURE.md`, `docs/VOICE_SPEC.md` y `docs/PLAN.md`.

mod editor;
pub mod params;

use std::sync::Arc;

use khodes_dsp::{Engine, Event as DspEvent, EventBody as DspEventBody, Params as DspParams};
use truce::prelude64::*;

pub use params::KhodesParams;

/// Número máximo de eventos MIDI traducidos por bloque.
///
/// El buffer es stack-allocated para no asignar memoria en `process`.
const MAX_EVENTS_PER_BLOCK: usize = 256;

/// Estado de producto de Khodes.
#[derive(Default)]
pub struct Khodes {
    engine: Engine,
    output_l: Vec<f64>,
}

impl PluginLogic for Khodes {
    type Params = KhodesParams;
    type DspState = Self;

    fn init(_: &Self::Params, _: &truce::core::tasks::InitContext) -> Self::DspState {
        Self::default()
    }

    fn reset(state: &mut Self::DspState, _: &Self::Params, config: &AudioConfig) {
        state
            .engine
            .prepare(config.sample_rate, config.max_block_size);
        state.output_l.resize(config.max_block_size, 0.0);
    }

    fn process(
        state: &mut Self::DspState,
        params: &Self::Params,
        buffer: &mut AudioBuffer,
        events: &EventList,
        _: &mut ProcessContext,
    ) -> ProcessStatus {
        let frames = buffer.num_samples();
        let channels = buffer.channels();
        if frames == 0 || channels == 0 || frames > state.output_l.len() {
            return ProcessStatus::Normal;
        }

        let dsp_params: DspParams = params.dsp_params();
        let mut translated = [DspEvent::new(0, DspEventBody::AllNotesOff); MAX_EVENTS_PER_BLOCK];
        let mut count = 0;

        for event in events.iter() {
            let offset = event.sample_offset as usize;
            if offset >= frames || count == translated.len() {
                continue;
            }
            let body = match event.body {
                EventBody::NoteOn { note, velocity, .. } => Some(DspEventBody::NoteOn {
                    note,
                    velocity: f64::from(velocity) / 127.0,
                }),
                EventBody::NoteOff { note, .. } => Some(DspEventBody::NoteOff { note }),
                EventBody::NoteOn2 { note, velocity, .. } => Some(DspEventBody::NoteOn {
                    note,
                    velocity: f64::from(velocity) / 65_535.0,
                }),
                EventBody::NoteOff2 { note, .. } => Some(DspEventBody::NoteOff { note }),
                EventBody::PitchBend { value, .. } => {
                    let normalized = (f64::from(value) - 8192.0) / 8192.0;
                    Some(DspEventBody::PitchBend {
                        semitones: normalized * f64::from(dsp_params.bend_range),
                    })
                }
                EventBody::ControlChange { cc, value, .. } => match cc {
                    1 => Some(DspEventBody::ModWheel {
                        amount: f64::from(value) / 127.0,
                    }),
                    64 => Some(DspEventBody::Sustain {
                        enabled: value >= 64,
                    }),
                    120 | 121 | 123 => Some(DspEventBody::AllNotesOff),
                    _ => None,
                },
                _ => None,
            };
            if let Some(body) = body {
                translated[count] = DspEvent::new(offset, body);
                count += 1;
            }
        }

        state.engine.process(
            &mut state.output_l[..frames],
            &translated[..count],
            &dsp_params,
        );

        {
            let (_, output) = buffer.io(0);
            output.copy_from_slice(&state.output_l[..frames]);
        }
        for channel in 1..channels {
            let (_, output) = buffer.io(channel);
            // M2 suma mono; M4 hará estéreo en el bus global.
            output.copy_from_slice(&state.output_l[..frames]);
        }

        ProcessStatus::Normal
    }

    fn editor(params: Arc<Self::Params>) -> Box<dyn Editor> {
        editor::create(params)
    }
}

truce::plugin! {
    logic: Khodes,
    params: KhodesParams,
}

truce::enable_rt_paranoid!();
