//! Esquema de parámetros de host de Khodes.
//!
//! Los IDs `1..19` son contrato publicado: no se renumeran ni se reutilizan.
//! Todos los valores que ve el host están normalizados `0..1`; la traducción a
//! unidades reales vive en el DSP.
//!
//! Ver `docs/PARAMETERS.md` y `docs/CONTRACTS.md`.

use truce::prelude64::*;

/// Parámetros de host de Khodes v1.
///
/// Los defaults son los del preset **Tine Classic**, expresados en el espacio
/// normalizado `0..1` que documenta `docs/PARAMETERS.md`.
#[derive(Params)]
pub struct KhodesParams {
    // Voz
    #[param(
        id = 1,
        name = "Brightness",
        group = "Voice",
        range = "linear(0, 1)",
        default = 1.0
    )]
    brightness: FloatParam,
    #[param(
        id = 2,
        name = "Key Click",
        group = "Voice",
        range = "linear(0, 1)",
        default = 0.0
    )]
    key_click: FloatParam,

    // Envelope
    #[param(
        id = 3,
        name = "Attack",
        group = "Envelope",
        range = "linear(0, 1)",
        default = 0.020100502512562814
    )]
    attack: FloatParam,
    #[param(
        id = 4,
        name = "Decay",
        group = "Envelope",
        range = "linear(0, 1)",
        default = 0.07035175879396985
    )]
    decay: FloatParam,
    #[param(
        id = 5,
        name = "Sustain",
        group = "Envelope",
        range = "linear(0, 1)",
        default = 0.6
    )]
    sustain: FloatParam,
    #[param(
        id = 6,
        name = "Release",
        group = "Envelope",
        range = "linear(0, 1)",
        default = 0.09547738693467336
    )]
    release: FloatParam,

    // Amplificador
    #[param(
        id = 7,
        name = "Volume",
        group = "Amp",
        range = "linear(0, 1)",
        default = 0.3
    )]
    volume: FloatParam,
    #[param(
        id = 8,
        name = "Drive",
        group = "Amp",
        range = "linear(0, 1)",
        default = 0.0
    )]
    drive: FloatParam,
    #[param(
        id = 9,
        name = "Bass",
        group = "Amp",
        range = "linear(0, 1)",
        default = 0.5
    )]
    bass: FloatParam,
    #[param(
        id = 10,
        name = "Treble",
        group = "Amp",
        range = "linear(0, 1)",
        default = 0.5
    )]
    treble: FloatParam,

    // Efectos
    #[param(
        id = 11,
        name = "Tremolo Rate",
        group = "FX",
        range = "linear(0, 1)",
        default = 0.4
    )]
    tremolo_rate: FloatParam,
    #[param(
        id = 12,
        name = "Tremolo Depth",
        group = "FX",
        range = "linear(0, 1)",
        default = 0.0
    )]
    tremolo_depth: FloatParam,
    #[param(
        id = 13,
        name = "Chorus Rate",
        group = "FX",
        range = "linear(0, 1)",
        default = 0.16
    )]
    chorus_rate: FloatParam,
    #[param(
        id = 14,
        name = "Chorus Depth",
        group = "FX",
        range = "linear(0, 1)",
        default = 0.0
    )]
    chorus_depth: FloatParam,
    #[param(
        id = 15,
        name = "Reverb",
        group = "FX",
        range = "linear(0, 1)",
        default = 0.0
    )]
    reverb: FloatParam,

    // Global
    #[param(
        id = 16,
        name = "Polyphony",
        group = "Global",
        range = "linear(0, 1)",
        default = 0.4838709677419355
    )]
    polyphony: FloatParam,
    #[param(
        id = 17,
        name = "Bend Range",
        group = "Global",
        range = "linear(0, 1)",
        default = 0.08333333333333333
    )]
    pitch_bend_range: FloatParam,
    #[param(
        id = 18,
        name = "Velocity",
        group = "Global",
        range = "linear(0, 1)",
        default = 0.5
    )]
    velocity_sensitivity: FloatParam,
    #[param(
        id = 19,
        name = "Fine Tune",
        group = "Global",
        range = "linear(0, 1)",
        default = 0.5
    )]
    fine_tune: FloatParam,

    // Telemetría audio → UI. No son parámetros de host: el derive les asigna
    // IDs en el rango de meters y no entran en el estado ni en el conteo de
    // parámetros (docs/adr/0003-dsp-ui-separation.md).
    #[meter]
    pub meter_peak_l: MeterSlot,
    #[meter]
    pub meter_peak_r: MeterSlot,
    #[meter]
    pub meter_voices: MeterSlot,
    #[meter]
    pub meter_note: MeterSlot,
    #[meter]
    pub meter_sustain: MeterSlot,
}

/// Rango real de cada parámetro, en unidades de producto.
///
/// El DSP usa estos rangos para convertir el valor normalizado del host.
pub mod ranges {
    pub const BRIGHTNESS_HZ: (f64, f64) = (500.0, 20_000.0);
    pub const KEY_CLICK: (f64, f64) = (0.0, 1.0);
    pub const ATTACK_S: (f64, f64) = (0.001, 0.2);
    pub const DECAY_S: (f64, f64) = (0.01, 2.0);
    pub const SUSTAIN: (f64, f64) = (0.0, 1.0);
    pub const RELEASE_S: (f64, f64) = (0.01, 2.0);
    pub const VOLUME: (f64, f64) = (0.0, 1.0);
    pub const DRIVE: (f64, f64) = (0.0, 1.0);
    pub const EQ_DB: (f64, f64) = (-12.0, 12.0);
    pub const TREMOLO_RATE_HZ: (f64, f64) = (0.0, 10.0);
    pub const TREMOLO_DEPTH: (f64, f64) = (0.0, 1.0);
    pub const CHORUS_RATE_HZ: (f64, f64) = (0.0, 5.0);
    pub const CHORUS_DEPTH: (f64, f64) = (0.0, 1.0);
    pub const REVERB: (f64, f64) = (0.0, 1.0);
    pub const POLYPHONY: (f64, f64) = (1.0, 32.0);
    pub const PITCH_BEND_SEMITONES: (f64, f64) = (0.0, 24.0);
    pub const VELOCITY_SENSITIVITY: (f64, f64) = (0.0, 1.0);
    pub const FINE_TUNE_CENTS: (f64, f64) = (-100.0, 100.0);
}

/// Convierte un valor normalizado `0..1` a un rango real lineal.
#[must_use]
pub fn denormalize(value: f64, (min, max): (f64, f64)) -> f64 {
    min + (max - min) * value.clamp(0.0, 1.0)
}

/// Convierte un valor real de un rango lineal a `0..1`.
#[must_use]
pub fn normalize(value: f64, (min, max): (f64, f64)) -> f64 {
    if (max - min).abs() < f64::EPSILON {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

/// Brightness sigue la curva logarítmica del contrato: `f = 500 * 40^n`.
#[must_use]
pub fn brightness_hz(normalized: f64) -> f64 {
    ranges::BRIGHTNESS_HZ.0
        * (ranges::BRIGHTNESS_HZ.1 / ranges::BRIGHTNESS_HZ.0).powf(normalized.clamp(0.0, 1.0))
}

/// Inversa exacta de [`brightness_hz`].
#[must_use]
pub fn normalize_brightness(hz: f64) -> f64 {
    let ratio = hz / ranges::BRIGHTNESS_HZ.0;
    (ratio.ln() / (ranges::BRIGHTNESS_HZ.1 / ranges::BRIGHTNESS_HZ.0).ln()).clamp(0.0, 1.0)
}

/// Polyphony es discreto: `round(1 + 31n)`, en `1..=32`.
#[must_use]
pub fn polyphony_voices(normalized: f64) -> u8 {
    (denormalize(normalized, ranges::POLYPHONY)).round() as u8
}

/// Bend Range es discreto: `round(24n)`, en `0..=24`.
#[must_use]
pub fn bend_range_semitones(normalized: f64) -> u8 {
    (denormalize(normalized, ranges::PITCH_BEND_SEMITONES)).round() as u8
}

/// Formatea un valor normalizado para el readout del editor.
///
/// Único punto de formato: Slint recibe cadenas ya listas y no duplica las
/// curvas de `docs/PARAMETERS.md`. Los IDs son el contrato publico `1..19`.
#[must_use]
pub fn display_value(id: u32, normalized: f64) -> String {
    let n = normalized.clamp(0.0, 1.0);
    let pct = || format!("{:.0} %", denormalize(n, (0.0, 1.0)) * 100.0);
    let db = || {
        let v = denormalize(n, ranges::EQ_DB);
        if v.abs() < 0.05 {
            "0.0 dB".to_string()
        } else {
            format!("{:+.1} dB", v)
        }
    };
    let seconds = |range: (f64, f64)| {
        let v = denormalize(n, range);
        if v < 1.0 {
            format!("{:.0} ms", v * 1000.0)
        } else {
            format!("{:.2} s", v)
        }
    };
    match id {
        1 => {
            let hz = brightness_hz(n);
            if hz >= 1000.0 {
                format!("{:.2} kHz", hz / 1000.0)
            } else {
                format!("{:.0} Hz", hz)
            }
        }
        2 | 5 | 7 | 8 | 12 | 14 | 15 | 18 => pct(),
        3 => seconds(ranges::ATTACK_S),
        4 => seconds(ranges::DECAY_S),
        6 => seconds(ranges::RELEASE_S),
        9 | 10 => db(),
        11 => format!("{:.1} Hz", denormalize(n, ranges::TREMOLO_RATE_HZ)),
        13 => format!("{:.2} Hz", denormalize(n, ranges::CHORUS_RATE_HZ)),
        16 => format!("{}", polyphony_voices(n)),
        17 => format!("{} st", bend_range_semitones(n)),
        19 => {
            let cents = denormalize(n, ranges::FINE_TUNE_CENTS);
            if cents.abs() < 0.5 {
                "0 ct".to_string()
            } else {
                format!("{:+.0} ct", cents)
            }
        }
        _ => String::new(),
    }
}

impl KhodesParams {
    /// Traduce los 19 valores normalizados del host a unidades del DSP.
    ///
    /// Las curvas y los redondeos son los de `docs/PARAMETERS.md`; este es el
    /// único punto de traducción host → motor.
    #[must_use]
    pub fn dsp_params(&self) -> khodes_dsp::Params {
        let value = |param: &FloatParam| param.value();
        khodes_dsp::Params {
            brightness_hz: brightness_hz(value(&self.brightness)),
            key_click: value(&self.key_click),
            attack_s: denormalize(value(&self.attack), ranges::ATTACK_S),
            decay_s: denormalize(value(&self.decay), ranges::DECAY_S),
            sustain: value(&self.sustain),
            release_s: denormalize(value(&self.release), ranges::RELEASE_S),
            volume: value(&self.volume),
            drive: value(&self.drive),
            bass_db: denormalize(value(&self.bass), ranges::EQ_DB),
            treble_db: denormalize(value(&self.treble), ranges::EQ_DB),
            tremolo_rate_hz: denormalize(value(&self.tremolo_rate), ranges::TREMOLO_RATE_HZ),
            tremolo_depth: value(&self.tremolo_depth),
            chorus_rate_hz: denormalize(value(&self.chorus_rate), ranges::CHORUS_RATE_HZ),
            chorus_depth: value(&self.chorus_depth),
            reverb: value(&self.reverb),
            polyphony: polyphony_voices(value(&self.polyphony)),
            bend_range: bend_range_semitones(value(&self.pitch_bend_range)),
            velocity_sensitivity: value(&self.velocity_sensitivity),
            fine_tune_cents: denormalize(value(&self.fine_tune), ranges::FINE_TUNE_CENTS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_value_uses_the_documented_units() {
        assert_eq!(display_value(1, 1.0), "20.00 kHz");
        assert_eq!(display_value(1, 0.0), "500 Hz");
        assert_eq!(display_value(3, 0.020100502512562814), "5 ms");
        assert_eq!(display_value(4, 0.07035175879396985), "150 ms");
        assert_eq!(display_value(9, 0.0), "-12.0 dB");
        assert_eq!(display_value(9, 0.5), "0.0 dB");
        assert_eq!(display_value(9, 1.0), "+12.0 dB");
        assert_eq!(display_value(11, 0.4), "4.0 Hz");
        assert_eq!(display_value(16, 0.4838709677419355), "16");
        assert_eq!(display_value(17, 0.08333333333333333), "2 st");
        assert_eq!(display_value(19, 0.5), "0 ct");
        assert_eq!(display_value(7, 0.3), "30 %");
    }

    #[test]
    fn normalized_defaults_match_the_contract() {
        assert!((brightness_hz(1.0) - 20_000.0).abs() < 1e-9);
        assert!((brightness_hz(0.0) - 500.0).abs() < 1e-9);
        assert!((brightness_hz(0.5) - 500.0 * 40.0_f64.sqrt()).abs() < 1e-9);
        assert_eq!(polyphony_voices(0.4838709677419355), 16);
        assert_eq!(bend_range_semitones(0.08333333333333333), 2);
        assert!((denormalize(0.020100502512562814, ranges::ATTACK_S) - 0.005).abs() < 1e-9);
        assert!((denormalize(0.07035175879396985, ranges::DECAY_S) - 0.15).abs() < 1e-9);
        assert!((denormalize(0.6, ranges::SUSTAIN) - 0.6).abs() < 1e-9);
        assert!((denormalize(0.09547738693467336, ranges::RELEASE_S) - 0.2).abs() < 1e-9);
        assert!((denormalize(0.5, ranges::EQ_DB) - 0.0).abs() < 1e-9);
        assert!((denormalize(0.4, ranges::TREMOLO_RATE_HZ) - 4.0).abs() < 1e-9);
        assert!((denormalize(0.16, ranges::CHORUS_RATE_HZ) - 0.8).abs() < 1e-9);
        assert!((denormalize(0.4838709677419355, ranges::POLYPHONY).round() - 16.0).abs() < 1e-9);
        assert!(
            (denormalize(0.08333333333333333, ranges::PITCH_BEND_SEMITONES).round() - 2.0).abs()
                < 1e-9
        );
        assert!((denormalize(0.5, ranges::FINE_TUNE_CENTS) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn host_defaults_translate_to_dsp_defaults() {
        let params = KhodesParams::default();
        assert_eq!(params.dsp_params(), khodes_dsp::Params::default());
    }

    #[test]
    fn normalize_round_trips() {
        for (value, range) in [
            (0.005, ranges::ATTACK_S),
            (-12.0, ranges::EQ_DB),
            (12.0, ranges::EQ_DB),
        ] {
            let round_trip = denormalize(normalize(value, range), range);
            assert!((round_trip - value).abs() < 1e-9);
        }
        for hz in [500.0, 1000.0, 3162.2776601683795, 20_000.0] {
            let round_trip = brightness_hz(normalize_brightness(hz));
            assert!((round_trip - hz).abs() < 1e-6);
        }
    }
}
