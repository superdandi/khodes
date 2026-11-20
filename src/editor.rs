//! Editor Slint de Khodes.
//!
//! En M1 solo valida el ciclo de vida del editor. Los controles reales se
//! implementan en M5 según `docs/UI_SPEC.md`.

use std::sync::Arc;

use truce::prelude::Editor;
use truce_slint::SlintEditor;
use truce_slint::slint::include_modules;

use crate::params::KhodesParams;

include_modules!();

pub fn create(params: Arc<KhodesParams>) -> Box<dyn Editor> {
    Box::new(
        SlintEditor::new(params, (1120, 700), |_state| {
            let ui = KhodesUi::new().expect("create Khodes Slint editor");
            Box::new(move |_context| {
                // M5 conectará aquí el snapshot de parámetros y telemetría.
                let _ = &ui;
            })
        })
        .resizable(true)
        .min_size((920, 620)),
    )
}
