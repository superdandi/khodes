//! Núcleo DSP puro de Khodes.
//!
//! Este crate no conoce TRUCE, Slint, el host ni el sistema de archivos.
//! Recibe eventos y parámetros ya traducidos y produce audio.
//!
//! Ver `docs/ARCHITECTURE.md` y `docs/VOICE_SPEC.md`.

#![forbid(unsafe_code)]

/// Versión del contrato de sonido.
///
/// Cambia solo cuando el audio cambia de forma intencional. Es independiente
/// de la versión del producto y de la versión del códec de presets.
pub const DSP_CONTRACT_VERSION: u32 = 1;
