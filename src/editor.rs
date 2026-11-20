//! Editor Slint de Khodes (M5).
//!
//! `ui/main.slint` solo muestra estado y emite intenciones. Este módulo es el
//! único que conoce TRUCE: traduce cada callback de la UI a un gesto de
//! automatización (`begin_edit` / `set_param` / `end_edit`) y sincroniza los
//! valores del host y la telemetría atómica en cada frame.
//!
//! Ver `docs/UI_SPEC.md` y `docs/adr/0003-dsp-ui-separation.md`.

use std::sync::Arc;

use truce::core::editor::PluginContextReadF64;
use truce::prelude::Editor;
use truce_slint::slint::include_modules;
use truce_slint::{PluginContext, SlintEditor};

use crate::params::KhodesParams;

include_modules!();

/// Closure de sincronización que `SlintEditor` llama en cada frame.
type SyncFn = Box<dyn Fn(&PluginContext<KhodesParams>)>;

/// Presets de fábrica del editor, en el orden del `ComboBox` de Slint.
/// Cada fila son los 19 valores normalizados de `docs/PARAMETERS.md`.
/// M6 migrará esta tabla a `khodes-kit` como presets `.khodespreset`.
const FACTORY_PRESETS: [[f64; 19]; 4] = [
    // Init: defaults del contrato.
    [
        1.0,
        0.0,
        0.020100502512562814,
        0.07035175879396985,
        0.6,
        0.09547738693467336,
        0.3,
        0.0,
        0.5,
        0.5,
        0.4,
        0.0,
        0.16,
        0.0,
        0.0,
        0.4838709677419355,
        0.08333333333333333,
        0.5,
        0.5,
    ],
    // Bright.
    [
        1.0,
        0.1,
        0.01,
        0.15,
        0.5,
        0.12,
        0.35,
        0.15,
        0.45,
        0.6,
        0.4,
        0.05,
        0.16,
        0.1,
        0.15,
        0.4838709677419355,
        0.08333333333333333,
        0.6,
        0.5,
    ],
    // Warm.
    [
        0.55,
        0.0,
        0.03,
        0.1,
        0.75,
        0.2,
        0.3,
        0.05,
        0.6,
        0.4,
        0.3,
        0.1,
        0.12,
        0.15,
        0.25,
        0.4838709677419355,
        0.08333333333333333,
        0.45,
        0.5,
    ],
    // Punch.
    [
        0.9,
        0.25,
        0.005,
        0.05,
        0.3,
        0.08,
        0.4,
        0.35,
        0.55,
        0.55,
        0.5,
        0.0,
        0.2,
        0.0,
        0.05,
        0.4838709677419355,
        0.08333333333333333,
        0.7,
        0.5,
    ],
];

/// Conecta el callback `changed` de un control con `set_param` del host.
/// El gesto `begin`/`end` lo emite la UI por separado (un solo par de callbacks
/// compartido por los 19 controles, con el ID del parámetro como argumento).
macro_rules! wire {
    ($ui:expr, $state:expr, $id:literal, $callback:ident) => {{
        let state = $state.clone();
        $ui.$callback(Box::new(move |value: f32| {
            state.set_param($id as u32, f64::from(value));
        }));
    }};
}

/// Sincroniza un parámetro y su readout en un frame.
macro_rules! sync {
    ($ui:expr, $state:expr, $id:literal, $property:ident, $text:ident) => {{
        let value = $state.get_param($id as u32);
        $ui.$property(value as f32);
        $ui.$text(crate::params::display_value($id as u32, value).into());
    }};
}

/// Origen de la telemetría mostrada por el editor.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeterSource {
    /// Lectura real de los slots atómicos publicados por `process`.
    Live,
    /// Valores fijos para la captura de nota activa (`slint-active.png`).
    Demo,
}

pub fn create(params: Arc<KhodesParams>) -> Box<dyn Editor> {
    create_at(params, (1120, 700))
}

/// Crea el editor con un tamaño explícito. Útil para las capturas baseline de
/// `docs/UI_SPEC.md` §10 (default, mínimo y wide).
pub(crate) fn create_at(params: Arc<KhodesParams>, size: (u32, u32)) -> Box<dyn Editor> {
    create_with_meters(params, size, MeterSource::Live)
}

/// Constructor usado por el generador de screenshots: permite fijar la
/// telemetría para la captura de nota activa sin tocar el hilo de audio.
pub(crate) fn create_with_meters(
    params: Arc<KhodesParams>,
    size: (u32, u32),
    source: MeterSource,
) -> Box<dyn Editor> {
    Box::new(
        SlintEditor::new(params, size, move |state| setup(state, source))
            .resizable(true)
            .min_size((920, 620)),
    )
}

/// Construye la UI y devuelve el closure de sincronización por frame.
fn setup(state: PluginContext<KhodesParams>, source: MeterSource) -> SyncFn {
    let ui = KhodesUi::new().expect("create Khodes Slint editor");

    // UI → host. Los controles incluyen el ID del parámetro en los callbacks
    // de gesto, así que basta una conexión para los 19.
    let begin = state.clone();
    ui.on_begin_edit(move |id| begin.begin_edit(id as u32));
    let end = state.clone();
    ui.on_end_edit(move |id| end.end_edit(id as u32));

    // El combo aplica presets de fábrica como 19 gestos completos. No abre
    // archivos: los presets completos del host se cargan desde el host (AGENTS).
    let preset = state.clone();
    ui.on_preset_changed(move |index| {
        if let Some(values) = FACTORY_PRESETS.get(index as usize) {
            for (i, value) in values.iter().enumerate() {
                preset.automate((i + 1) as u32, *value);
            }
        }
    });

    wire!(ui, state, 1, on_brightness_changed);
    wire!(ui, state, 2, on_key_click_changed);
    wire!(ui, state, 3, on_attack_changed);
    wire!(ui, state, 4, on_decay_changed);
    wire!(ui, state, 5, on_sustain_changed);
    wire!(ui, state, 6, on_release_changed);
    wire!(ui, state, 7, on_volume_changed);
    wire!(ui, state, 8, on_drive_changed);
    wire!(ui, state, 9, on_bass_changed);
    wire!(ui, state, 10, on_treble_changed);
    wire!(ui, state, 11, on_trem_rate_changed);
    wire!(ui, state, 12, on_trem_depth_changed);
    wire!(ui, state, 13, on_chorus_rate_changed);
    wire!(ui, state, 14, on_chorus_depth_changed);
    wire!(ui, state, 15, on_reverb_changed);
    wire!(ui, state, 16, on_polyphony_changed);
    wire!(ui, state, 17, on_bend_range_changed);
    wire!(ui, state, 18, on_velocity_changed);
    wire!(ui, state, 19, on_fine_tune_changed);

    // Los meter IDs viven en el espacio de meters, no en 1..19.
    let meter_peak_l = state.params().meter_peak_l.id();
    let meter_peak_r = state.params().meter_peak_r.id();
    let meter_voices = state.params().meter_voices.id();
    let meter_note = state.params().meter_note.id();
    let meter_sustain = state.params().meter_sustain.id();

    Box::new(move |ctx: &PluginContext<KhodesParams>| {
        sync!(ui, ctx, 1, set_brightness, set_brightness_text);
        sync!(ui, ctx, 2, set_key_click, set_key_click_text);
        sync!(ui, ctx, 3, set_attack, set_attack_text);
        sync!(ui, ctx, 4, set_decay, set_decay_text);
        sync!(ui, ctx, 5, set_sustain, set_sustain_text);
        sync!(ui, ctx, 6, set_release, set_release_text);
        sync!(ui, ctx, 7, set_volume, set_volume_text);
        sync!(ui, ctx, 8, set_drive, set_drive_text);
        sync!(ui, ctx, 9, set_bass, set_bass_text);
        sync!(ui, ctx, 10, set_treble, set_treble_text);
        sync!(ui, ctx, 11, set_trem_rate, set_trem_rate_text);
        sync!(ui, ctx, 12, set_trem_depth, set_trem_depth_text);
        sync!(ui, ctx, 13, set_chorus_rate, set_chorus_rate_text);
        sync!(ui, ctx, 14, set_chorus_depth, set_chorus_depth_text);
        sync!(ui, ctx, 15, set_reverb, set_reverb_text);
        sync!(ui, ctx, 16, set_polyphony, set_polyphony_text);
        sync!(ui, ctx, 17, set_bend_range, set_bend_range_text);
        sync!(ui, ctx, 18, set_velocity, set_velocity_text);
        sync!(ui, ctx, 19, set_fine_tune, set_fine_tune_text);

        if source == MeterSource::Demo {
            // Telemetría determinista para `slint-active.png`: 4 voces,
            // nota y sustain activos, picos L/R distintos.
            ui.set_voices_text("voces: 4/16".into());
            ui.set_meter_voices(4.0 / 16.0);
            ui.set_meter_peak_l(0.72);
            ui.set_meter_peak_r(0.58);
            ui.set_meter_note(1.0);
            ui.set_meter_sustain(1.0);
        } else {
            let voices = (ctx.get_meter(meter_voices).clamp(0.0, 1.0) * 16.0).round() as u32;
            ui.set_voices_text(format!("voces: {voices}/16").into());
            ui.set_meter_peak_l(ctx.get_meter(meter_peak_l));
            ui.set_meter_peak_r(ctx.get_meter(meter_peak_r));
            ui.set_meter_note(ctx.get_meter(meter_note));
            ui.set_meter_sustain(ctx.get_meter(meter_sustain));
            ui.set_meter_voices(ctx.get_meter(meter_voices));
        }
    })
}
