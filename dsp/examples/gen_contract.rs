//! Genera el corpus congelado `contract/v1/` (docs/TESTING.md §4).
//!
//! Uso: `cargo run --locked --release -p khodes-dsp --example gen_contract`
//!
//! Los fixtures se escriben con bits f64 en hexadecimal, por lo que el output
//! es idéntico entre plataformas con IEEE 754. No se reescribe un corpus ya
//! congelado: para un cambio de sonido, crear `contract/v2/`.

use std::path::Path;

use khodes_dsp::contract_format::{Fixture, encode};
use khodes_dsp::{Event, EventBody, Params};

/// (nombre, sample rate, block size, parámetros, eventos, frames).
type Case = (&'static str, f64, usize, Params, Vec<Event>, usize);

fn note(note: u8) -> Event {
    Event::new(
        0,
        EventBody::NoteOn {
            channel: 0,
            note,
            velocity: 1.0,
            note_id: i32::from(note),
        },
    )
}

fn render(
    name: &str,
    sample_rate: f64,
    block_size: usize,
    params: Params,
    events: Vec<Event>,
    frames: usize,
) -> Fixture {
    let mut engine = khodes_dsp::Engine::default();
    engine.prepare(sample_rate, block_size);
    let mut left = vec![0.0; frames];
    let mut right = vec![0.0; frames];

    let mut pos = 0;
    while pos < frames {
        let end = (pos + block_size).min(frames);
        let block_events: Vec<Event> = events
            .iter()
            .filter(|e| e.sample_offset >= pos && e.sample_offset < end)
            .map(|e| Event::new(e.sample_offset - pos, e.body))
            .collect();
        engine.process_stereo(
            &mut left[pos..end],
            &mut right[pos..end],
            &block_events,
            &params,
        );
        pos = end;
    }

    Fixture {
        name: name.to_string(),
        sample_rate,
        block_size,
        params,
        events,
        left,
        right,
    }
}

fn main() {
    let dir = Path::new("contract/v1");
    std::fs::create_dir_all(dir).expect("crear contract/v1");

    let sr = 48_000.0;
    let steal_events: Vec<Event> = (36..68).map(note).collect();

    // Doce casos de docs/TESTING.md §4.1; el caso 12 tiene dos variantes.
    let cases: [Case; 13] = [
        (
            "c4_defaults",
            sr,
            256,
            Params::default(),
            vec![note(60)],
            2048,
        ),
        (
            "c4_drive",
            sr,
            256,
            Params {
                drive: 0.5,
                ..Params::default()
            },
            vec![note(60)],
            2048,
        ),
        (
            "c4_brightness_2k",
            sr,
            256,
            Params {
                brightness_hz: 2_000.0,
                ..Params::default()
            },
            vec![note(60)],
            2048,
        ),
        (
            "c4_chorus",
            sr,
            256,
            Params {
                chorus_rate_hz: 0.5,
                chorus_depth: 0.5,
                ..Params::default()
            },
            vec![note(60)],
            2048,
        ),
        (
            "c4_tremolo",
            sr,
            256,
            Params {
                tremolo_rate_hz: 5.0,
                tremolo_depth: 0.5,
                ..Params::default()
            },
            vec![note(60)],
            2048,
        ),
        (
            "c4_reverb",
            sr,
            256,
            Params {
                reverb: 0.5,
                ..Params::default()
            },
            vec![note(60)],
            12_000,
        ),
        (
            "chord_4",
            sr,
            256,
            Params::default(),
            vec![note(60), note(64), note(67), note(71)],
            4096,
        ),
        (
            "steal_32",
            sr,
            256,
            Params {
                polyphony: 8,
                ..Params::default()
            },
            steal_events,
            4096,
        ),
        (
            "sustain_off",
            sr,
            256,
            Params::default(),
            vec![
                Event::new(
                    0,
                    EventBody::Sustain {
                        channel: 0,
                        enabled: true,
                    },
                ),
                note(60),
                Event::new(
                    128,
                    EventBody::NoteOff {
                        channel: 0,
                        note: 60,
                        note_id: 60,
                    },
                ),
                Event::new(
                    512,
                    EventBody::Sustain {
                        channel: 0,
                        enabled: false,
                    },
                ),
            ],
            4096,
        ),
        (
            "pitch_bend",
            sr,
            256,
            Params::default(),
            vec![
                note(60),
                Event::new(512, EventBody::PitchBend { semitones: 2.0 }),
                Event::new(1024, EventBody::PitchBend { semitones: -2.0 }),
            ],
            4096,
        ),
        (
            "sr_96000",
            96_000.0,
            512,
            Params::default(),
            vec![note(60)],
            4096,
        ),
        ("block_1", sr, 1, Params::default(), vec![note(60)], 512),
        (
            "block_2048",
            sr,
            2048,
            Params::default(),
            vec![note(60)],
            4096,
        ),
    ];

    let count = cases.len();
    for (name, sample_rate, block_size, params, events, frames) in cases {
        let fixture = render(name, sample_rate, block_size, params, events, frames);
        let path = dir.join(format!("{name}.khdsp"));
        std::fs::write(&path, encode(&fixture)).expect("escribir fixture");
        println!("{}", path.display());
    }
    println!("{count} fixtures generados");
}
