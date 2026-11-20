//! Estado durable de Khodes que viaja en el bloque custom del host.
//!
//! Los 19 parámetros los persiste TRUCE automáticamente en el sobre; aquí solo
//! vive el nombre del preset. Este bloque se publica cada bloque de audio desde
//! `snapshot_into` y se restaura en `load_state`, que además cancela todo el
//! estado transitorio (ADR 0004, `CONTRACTS.md` §7).
//!
//! No contiene notas, sustain, bend, fases de LFO ni colas de reverb.

use truce::State;
use truce::core::custom_state::State as _;

/// Nombre del preset cuando el host no aporta metadata.
pub(crate) const DEFAULT_PRESET_NAME: &str = "Tine Classic";

/// Estado adicional guardado por el host junto a los parámetros.
#[derive(State, Default, Clone, Debug, PartialEq)]
pub(crate) struct PersistentState {
    /// Nombre del preset activo, mostrado por el editor.
    pub(crate) preset_name: String,
}

impl PersistentState {
    /// Estado inicial: el preset de fábrica por defecto.
    pub(crate) fn initial() -> Self {
        Self {
            preset_name: DEFAULT_PRESET_NAME.to_string(),
        }
    }

    /// Estado con un nombre concreto.
    pub(crate) fn named(name: impl Into<String>) -> Self {
        Self {
            preset_name: name.into(),
        }
    }

    /// Serializa para `snapshot_into` o para el canal `set_state` del editor.
    pub(crate) fn encode(&self) -> Vec<u8> {
        self.serialize()
    }

    /// Decodifica un bloque custom del host. `None` si está malformado.
    pub(crate) fn decode(data: &[u8]) -> Option<Self> {
        Self::deserialize(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_state_round_trips_the_name() {
        let state = PersistentState::named("Suitcase Warm");
        let bytes = state.encode();
        assert_eq!(PersistentState::decode(&bytes), Some(state));
    }

    #[test]
    fn malformed_state_is_rejected_without_changing_anything() {
        assert_eq!(PersistentState::decode(&[]), None);
        assert_eq!(PersistentState::decode(&[0xFF, 0xFF, 0xFF, 0xFF]), None);
    }

    #[test]
    fn encoding_is_byte_stable() {
        let state = PersistentState::named("Bell Air");
        assert_eq!(state.encode(), state.encode());
    }
}
