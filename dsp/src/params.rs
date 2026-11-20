//! Parámetros del motor en unidades reales.
//!
//! El wrapper `src/` convierte los 19 valores normalizados del host a estas
//! unidades. El DSP no conoce IDs de parámetro ni rangos del host.
//!
//! Ver `docs/PARAMETERS.md`.

/// Los 19 parámetros de Khodes, ya traducidos a unidades de producto.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Params {
    pub brightness_hz: f64,
    pub key_click: f64,
    pub attack_s: f64,
    pub decay_s: f64,
    pub sustain: f64,
    pub release_s: f64,
    pub volume: f64,
    pub drive: f64,
    pub bass_db: f64,
    pub treble_db: f64,
    pub tremolo_rate_hz: f64,
    pub tremolo_depth: f64,
    pub chorus_rate_hz: f64,
    pub chorus_depth: f64,
    pub reverb: f64,
    pub polyphony: u8,
    pub bend_range: u8,
    pub velocity_sensitivity: f64,
    pub fine_tune_cents: f64,
}

impl Default for Params {
    /// Defaults del preset **Tine Classic** (`docs/PARAMETERS.md` §4).
    fn default() -> Self {
        Self {
            brightness_hz: 20_000.0,
            key_click: 0.0,
            attack_s: 0.005,
            decay_s: 0.15,
            sustain: 0.6,
            release_s: 0.2,
            volume: 0.3,
            drive: 0.0,
            bass_db: 0.0,
            treble_db: 0.0,
            tremolo_rate_hz: 4.0,
            tremolo_depth: 0.0,
            chorus_rate_hz: 0.8,
            chorus_depth: 0.0,
            reverb: 0.0,
            polyphony: 16,
            bend_range: 2,
            velocity_sensitivity: 0.5,
            fine_tune_cents: 0.0,
        }
    }
}

impl Params {
    /// Sustituye los valores por otros válidos, sin asignar memoria.
    pub fn update(&mut self, other: &Self) {
        *self = *other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_published_contract() {
        let p = Params::default();
        assert_eq!(p.brightness_hz, 20_000.0);
        assert_eq!(p.attack_s, 0.005);
        assert_eq!(p.decay_s, 0.15);
        assert_eq!(p.sustain, 0.6);
        assert_eq!(p.release_s, 0.2);
        assert_eq!(p.volume, 0.3);
        assert_eq!(p.polyphony, 16);
        assert_eq!(p.bend_range, 2);
        assert_eq!(p.velocity_sensitivity, 0.5);
        assert_eq!(p.fine_tune_cents, 0.0);
    }
}
