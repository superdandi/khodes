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

/// Crea el editor con un tamaño explícito para las capturas baseline de
/// `docs/UI_SPEC.md` §10. El editor normal se construye vía `PluginLogic::editor`.
#[doc(hidden)]
pub fn editor_at(params: Arc<KhodesParams>, size: (u32, u32)) -> Box<dyn Editor> {
    editor::create_at(params, size)
}

/// Variante de `editor_at` con telemetría fija para la captura de nota activa.
#[doc(hidden)]
pub fn editor_at_with_demo_meters(params: Arc<KhodesParams>, size: (u32, u32)) -> Box<dyn Editor> {
    editor::create_with_meters(params, size, editor::MeterSource::Demo)
}

/// Estado de producto de Khodes.
#[derive(Default)]
pub struct Khodes {
    engine: Engine,
    left: Vec<f64>,
    right: Vec<f64>,
    mapper: MidiMapper,
    /// Último estado de sustain visto en el bloque (para el LED de la UI).
    sustain: bool,
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
        ctx: &mut ProcessContext,
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

        // Telemetría para el editor. `ctx.set_meter` escribe en slots atómicos;
        // nunca bloquea ni asigna en el hilo de audio (ADR 0003). Se normaliza
        // el estado de sustain desde los eventos ya traducidos, sin escanear
        // la lista de eventos del host otra vez.
        for event in &translated[..count] {
            if let DspEventBody::Sustain { enabled, .. } = event.body {
                state.sustain = enabled;
            }
        }
        let mut peak_l = 0.0_f32;
        let mut peak_r = 0.0_f32;
        for i in 0..frames {
            peak_l = peak_l.max(state.left[i].abs() as f32);
            peak_r = peak_r.max(state.right[i].abs() as f32);
        }
        let voices = state.engine.active_voices() as f32;
        ctx.set_meter(params.meter_peak_l.id(), peak_l.min(1.0));
        ctx.set_meter(params.meter_peak_r.id(), peak_r.min(1.0));
        // 16 es la polifonía por defecto del contrato; la UI muestra N/16.
        ctx.set_meter(params.meter_voices.id(), (voices / 16.0).min(1.0));
        ctx.set_meter(params.meter_note.id(), if voices > 0.0 { 1.0 } else { 0.0 });
        ctx.set_meter(
            params.meter_sustain.id(),
            if state.sustain { 1.0 } else { 0.0 },
        );

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
