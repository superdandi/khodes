//! Genera los baselines visuales del editor (`docs/UI_SPEC.md` §10).
//!
//! ```sh
//! cargo run --example screenshots
//! ```
//!
//! Las cuatro capturas salen en `screenshots/`. La ruta headless usa el
//! `SoftwareRenderer` de Slint a través de `Editor::screenshot`, sin GPU ni
//! ventana. No se abre ningún diálogo de archivo.

use std::{fs, sync::Arc};

use khodes::{KhodesParams, editor_at, editor_at_with_demo_meters};

fn capture(name: &str, size: (u32, u32), demo: bool) {
    let params = Arc::new(KhodesParams::default());
    let mut editor = if demo {
        editor_at_with_demo_meters(Arc::clone(&params), size)
    } else {
        editor_at(Arc::clone(&params), size)
    };

    let erased: Arc<dyn truce::params::Params> = params;
    let (rgba, width, height) = editor
        .screenshot(erased)
        .expect("Slint headless screenshot must render");

    let image = image::RgbaImage::from_raw(width, height, rgba).expect("RGBA buffer size");
    fs::create_dir_all("screenshots").expect("create screenshots/");
    let path = format!("screenshots/{name}");
    image.save(&path).expect("save PNG");
    println!("{path}: {width}×{height}");
}

fn main() {
    capture("slint-default.png", (1120, 700), false);
    capture("slint-minimum.png", (920, 620), false);
    capture("slint-wide.png", (1440, 760), false);
    capture("slint-active.png", (1120, 700), true);
}
