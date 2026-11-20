//! Khodes — piano eléctrico de tine sintetizado.
//!
//! Este crate contiene la integración de producto: parámetros, MIDI, estado y
//! editor. El DSP vive en `khodes-dsp` y los presets en `khodes-kit`.
//!
//! Ver `docs/ARCHITECTURE.md`, `docs/VOICE_SPEC.md` y `docs/PLAN.md`.

mod editor;
mod midi;
pub mod params;
mod state;

use std::sync::Arc;

use khodes_dsp::{Engine, Event as DspEvent, EventBody as DspEventBody, Params as DspParams};
use midi::{MAX_EVENTS_PER_BLOCK, MidiMapper};
use state::PersistentState;
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
pub struct Khodes {
    engine: Engine,
    left: Vec<f64>,
    right: Vec<f64>,
    mapper: MidiMapper,
    /// Último estado de sustain visto en el bloque (para el LED de la UI).
    sustain: bool,
    /// Datos durables que el host guarda además de los 19 parámetros.
    persistent: PersistentState,
}

impl Default for Khodes {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
            left: Vec::new(),
            right: Vec::new(),
            mapper: MidiMapper::default(),
            sustain: false,
            persistent: PersistentState::initial(),
        }
    }
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

    /// Publica el bloque custom que el host guarda junto a los parámetros.
    /// TRUCE persiste los 19 IDs automáticamente; aquí solo viaja el nombre.
    /// `serialize_into` reutiliza la capacidad del buffer (RT-safe).
    fn snapshot_into(state: &Self::DspState, buf: &mut Vec<u8>) -> bool {
        state.persistent.serialize_into(buf);
        true
    }

    /// Restaura el nombre y cancela todo estado transitorio. TRUCE lo ejecuta
    /// en el hilo de audio entre bloques, con acceso exclusivo a `state`.
    /// Cargar un estado nunca reanuda notas (`CONTRACTS.md` §7).
    fn load_state(state: &mut Self::DspState, data: &[u8]) -> Result<(), StateLoadError> {
        let persistent = PersistentState::decode(data)
            .ok_or(StateLoadError::Malformed("Khodes/PersistentState"))?;
        state.persistent = persistent;
        // `Engine::reset` libera voces, descarta colas y centra bend/mod wheel
        // sin reasignar buffers (a diferencia de `prepare`).
        state.engine.reset();
        state.mapper.reset();
        state.sustain = false;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    const FRAMES: usize = 512;

    /// Estado con voz activa, sustain, bend y modulación, como si el usuario
    /// estuviera tocando cuando el host guarda/restaura el proyecto.
    fn playing_state() -> Khodes {
        let mut state = Khodes::default();
        state.engine.prepare(48_000.0, FRAMES);
        let mut left = vec![0.0; FRAMES];
        let mut right = vec![0.0; FRAMES];
        let events = [
            DspEvent::new(
                0,
                DspEventBody::NoteOn {
                    channel: 0,
                    note: 60,
                    velocity: 0.9,
                    note_id: 7,
                },
            ),
            DspEvent::new(0, DspEventBody::PitchBend { semitones: 2.0 }),
            DspEvent::new(0, DspEventBody::ModWheel { amount: 0.8 }),
            DspEvent::new(
                0,
                DspEventBody::Sustain {
                    channel: 0,
                    enabled: true,
                },
            ),
        ];
        state
            .engine
            .process_stereo(&mut left, &mut right, &events, &DspParams::default());
        state.sustain = true;
        assert_eq!(state.engine.active_voices(), 1);
        assert!(state.engine.bend_semitones().abs() > 0.0);
        state
    }

    #[test]
    fn load_state_cancels_notes_sustain_bend_and_modulation() {
        let mut state = playing_state();
        let blob = PersistentState::named("Bell Air").encode();

        Khodes::load_state(&mut state, &blob).expect("un blob válido debe cargar");

        assert_eq!(state.engine.active_voices(), 0, "no se reanudan notas");
        assert!(!state.sustain, "el pedal se considera suelto");
        assert_eq!(state.engine.bend_semitones(), 0.0, "bend vuelve a centro");
        assert_eq!(state.engine.mod_wheel(), 0.0);
        assert_eq!(state.persistent.preset_name, "Bell Air");
    }

    #[test]
    fn state_blob_has_no_performance_data() {
        let clean = Khodes::default();
        let playing = playing_state();
        assert_eq!(clean.persistent.encode(), playing.persistent.encode());
        assert_eq!(
            PersistentState::decode(&playing.persistent.encode()),
            Some(PersistentState::initial())
        );
    }

    #[test]
    fn snapshot_is_byte_stable_and_reuses_its_buffer() {
        let state = Khodes::default();
        let mut first = Vec::new();
        let mut second = vec![0xAA; 512];
        assert!(Khodes::snapshot_into(&state, &mut first));
        assert!(Khodes::snapshot_into(&state, &mut second));
        assert_eq!(first, second);
        assert!(Khodes::snapshot_into(&state, &mut second));
        assert_eq!(first, second, "el buffer se limpia; no acumula bytes");
        assert_eq!(
            PersistentState::decode(&first),
            Some(PersistentState::initial())
        );
    }

    #[test]
    fn malformed_state_is_rejected_without_changing_the_plugin() {
        let mut state = playing_state();
        let before = state.persistent.encode();
        assert!(Khodes::load_state(&mut state, &[0xFF; 16]).is_err());
        assert_eq!(state.persistent.encode(), before);
        assert_eq!(state.engine.active_voices(), 1, "un error no cancela notas");
    }

    #[test]
    fn factory_defaults_match_the_host_parameter_defaults() {
        let preset = khodes_kit::factory(0).expect("Tine Classic");
        let params = KhodesParams::default();
        for (index, expected) in preset.values.iter().enumerate() {
            let actual = truce::params::Params::get_normalized(&params, index as u32 + 1)
                .expect("ID publicado");
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "default del parámetro {}",
                index + 1
            );
        }
    }
}
