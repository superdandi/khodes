//! Baseline de CPU del motor con 16 voces sostenidas a 48 kHz.
//!
//! Uso: `cargo run --locked --release -p khodes-dsp --example cpu_bench`
//!
//! Mide el tiempo de render puro del DSP, sin wrapper ni host. No es una
//! medida de consumo en DAW: sirve como referencia reproducible entre cambios.
//! Cuatro casos: sin FX (comparable con el baseline M3), chorus, reverb y
//! ambos activos (M4-21).

use std::time::Instant;

use khodes_dsp::{Engine, Event, EventBody, Params};

const SAMPLE_RATE: f64 = 48_000.0;
const BLOCK: usize = 256;
const SECONDS: usize = 10;
const VOICES: u8 = 16;

fn events() -> Vec<Event> {
    (0..VOICES)
        .map(|i| {
            Event::new(
                0,
                EventBody::NoteOn {
                    channel: i % 16,
                    note: 48 + i,
                    velocity: 1.0,
                    note_id: i32::from(i),
                },
            )
        })
        .collect()
}

fn run_case(name: &str, params: &Params) {
    let mut engine = Engine::default();
    engine.prepare(SAMPLE_RATE, BLOCK);
    let events = events();

    let mut out = vec![0.0; BLOCK];
    let blocks = ((SAMPLE_RATE as usize) * SECONDS) / BLOCK;

    // Calienta cachés y ramas antes de medir.
    for _ in 0..64 {
        engine.process(&mut out, &[], params);
    }

    let start = Instant::now();
    for i in 0..blocks {
        let batch: &[Event] = if i == 0 { &events } else { &[] };
        engine.process(&mut out, batch, params);
        std::hint::black_box(&out);
    }
    let elapsed = start.elapsed();

    let audio_seconds = (blocks * BLOCK) as f64 / SAMPLE_RATE;
    let realtime_factor = audio_seconds / elapsed.as_secs_f64();
    let ms_per_block = elapsed.as_secs_f64() * 1000.0 / blocks as f64;

    println!("caso: {name}");
    println!("  voces: {VOICES}");
    println!("  sample rate: {SAMPLE_RATE:.0} Hz");
    println!("  audio renderizado: {audio_seconds:.2} s");
    println!("  tiempo de CPU: {:.3} s", elapsed.as_secs_f64());
    println!("  factor tiempo real: {realtime_factor:.1}x");
    println!("  ms por bloque de {BLOCK} frames: {ms_per_block:.4}");
}

fn main() {
    let base = Params {
        polyphony: VOICES,
        sustain: 1.0,
        ..Params::default()
    };

    run_case("fx_off", &base);

    run_case(
        "chorus_on",
        &Params {
            chorus_depth: 0.5,
            ..base
        },
    );

    run_case(
        "reverb_on",
        &Params {
            reverb: 0.5,
            ..base
        },
    );

    run_case(
        "all_on",
        &Params {
            chorus_depth: 0.5,
            reverb: 0.5,
            ..base
        },
    );
}
