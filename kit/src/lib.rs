//! Presets de fábrica y códec `.khodespreset`.
//!
//! Este crate define los datos durables de Khodes y valida el formato nativo.
//! No conoce el host, la UI ni el hilo de audio.
//!
//! Ver `docs/STATE_AND_PRESETS.md`.

#![forbid(unsafe_code)]

/// Versión del layout binario de `.khodespreset`.
///
/// Independiente de la versión del producto y de `DSP_CONTRACT_VERSION`.
pub const PRESET_VERSION: u32 = 1;
