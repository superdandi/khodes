//! Khodes — piano eléctrico de tine sintetizado.
//!
//! Este crate contiene la integración de producto: parámetros, MIDI, estado y
//! editor. El DSP vive en `khodes-dsp` y los presets en `khodes-kit`.
//!
//! Ver `docs/ARCHITECTURE.md`, `docs/VOICE_SPEC.md` y `docs/PLAN.md`.

mod editor;
mod midi;
pub mod params;

use std::sync::Arc;

use khodes_dsp::{Engine, Event as DspEvent, EventBody as DspEventBody, Params as DspParams};
use midi::{MAX_EVENTS_PER_BLOCK, MidiMapper};
use truce::prelude64::*;

pub use params::KhodesParams;

/// Estado de producto de Khodes.
#[derive(Default)]
pub struct Khodes {
    engine: Engine,
    left: Vec<f64>,
    right: Vec<f64>,
    mapper: MidiMapper,
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
        state.left.resize(config.max_block_size, 0.0);
        state.right.resize(config.max_block_size, 0.0);
        state.mapper.reset();
    }

    /// El oversampling 4x del drive introduce 1 muestra de latencia.
    fn latency(state: &Self::DspState) -> u32 {
        state.engine.reported_latency_samples() as u32
    }

    /// La cola de la reverb convolucional dura 2,5 s.
    fn tail(_: &Self::DspState) -> u32 {
        (2.5 * 48_000.0) as u32
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
        if frames == 0 || channels == 0 || frames > state.left.len() {
            return ProcessStatus::Normal;
        }

        let dsp_params: DspParams = params.dsp_params();
        let mut translated = [DspEvent::new(0, DspEventBody::AllNotesOff); MAX_EVENTS_PER_BLOCK];
        let count = state
            .mapper
            .translate(events, &mut translated, frames, dsp_params.bend_range);

        state.engine.process_stereo(
            &mut state.left[..frames],
            &mut state.right[..frames],
            &translated[..count],
            &dsp_params,
        );

        for channel in 0..channels {
            let (_, output) = buffer.io(channel);
            if channel == 0 {
                output.copy_from_slice(&state.left[..frames]);
            } else {
                output.copy_from_slice(&state.right[..frames]);
            }
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
