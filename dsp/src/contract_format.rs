//! Formato de fixtures del corpus de regresión (`docs/TESTING.md` §4).
//!
//! `.khdsp` es texto UTF-8 determinista. Los valores `f64` se guardan como bits
//! en hexadecimal (16 dígitos) para que el round-trip sea exacto y el SHA256
//! estable entre plataformas.

use crate::{Event, EventBody, Params};

/// Versión del formato de fixture, independiente de la versión del producto.
pub const FIXTURE_VERSION: u32 = 1;

/// Un caso del corpus de regresión.
#[derive(Clone, Debug)]
pub struct Fixture {
    pub name: String,
    pub sample_rate: f64,
    pub block_size: usize,
    pub params: Params,
    pub events: Vec<Event>,
    pub left: Vec<f64>,
    pub right: Vec<f64>,
}

fn encode_f64(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn decode_f64(text: &str) -> Option<f64> {
    u64::from_str_radix(text, 16).ok().map(f64::from_bits)
}

fn params_values(p: &Params) -> [f64; 19] {
    [
        p.brightness_hz,
        p.key_click,
        p.attack_s,
        p.decay_s,
        p.sustain,
        p.release_s,
        p.volume,
        p.drive,
        p.bass_db,
        p.treble_db,
        p.tremolo_rate_hz,
        p.tremolo_depth,
        p.chorus_rate_hz,
        p.chorus_depth,
        p.reverb,
        f64::from(p.polyphony),
        f64::from(p.bend_range),
        p.velocity_sensitivity,
        p.fine_tune_cents,
    ]
}

fn params_from_values(v: &[f64; 19]) -> Params {
    Params {
        brightness_hz: v[0],
        key_click: v[1],
        attack_s: v[2],
        decay_s: v[3],
        sustain: v[4],
        release_s: v[5],
        volume: v[6],
        drive: v[7],
        bass_db: v[8],
        treble_db: v[9],
        tremolo_rate_hz: v[10],
        tremolo_depth: v[11],
        chorus_rate_hz: v[12],
        chorus_depth: v[13],
        reverb: v[14],
        polyphony: v[15] as u8,
        bend_range: v[16] as u8,
        velocity_sensitivity: v[17],
        fine_tune_cents: v[18],
    }
}

fn event_line(event: &Event) -> String {
    let (kind, channel, note, note_id, a, b) = match event.body {
        EventBody::NoteOn {
            channel,
            note,
            velocity,
            note_id,
        } => (0, channel, note, note_id, velocity, 0.0),
        EventBody::NoteOff {
            channel,
            note,
            note_id,
        } => (1, channel, note, note_id, 0.0, 0.0),
        EventBody::PitchBend { semitones } => (2, 0, 0, 0, semitones, 0.0),
        EventBody::Sustain { channel, enabled } => {
            (3, channel, 0, 0, if enabled { 1.0 } else { 0.0 }, 0.0)
        }
        EventBody::ModWheel { amount } => (4, 0, 0, 0, amount, 0.0),
        EventBody::AllNotesOff => (5, 0, 0, 0, 0.0, 0.0),
        EventBody::AllSoundOff => (6, 0, 0, 0, 0.0, 0.0),
        EventBody::ResetControllers => (7, 0, 0, 0, 0.0, 0.0),
        // Extensión compatible: los fixtures v1 existentes no usan este kind.
        EventBody::MasterVolume { level } => (8, 0, 0, 0, level, 0.0),
    };
    format!(
        "{} {} {} {} {} {} {}",
        kind,
        event.sample_offset,
        channel,
        note,
        note_id,
        encode_f64(a),
        encode_f64(b)
    )
}

fn parse_event(line: &str) -> Option<Event> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 7 {
        return None;
    }
    let kind: u8 = parts[0].parse().ok()?;
    let offset: usize = parts[1].parse().ok()?;
    let channel: u8 = parts[2].parse().ok()?;
    let note: u8 = parts[3].parse().ok()?;
    let note_id: i32 = parts[4].parse().ok()?;
    let a = decode_f64(parts[5])?;
    let body = match kind {
        0 => EventBody::NoteOn {
            channel,
            note,
            velocity: a,
            note_id,
        },
        1 => EventBody::NoteOff {
            channel,
            note,
            note_id,
        },
        2 => EventBody::PitchBend { semitones: a },
        3 => EventBody::Sustain {
            channel,
            enabled: a > 0.5,
        },
        4 => EventBody::ModWheel { amount: a },
        5 => EventBody::AllNotesOff,
        6 => EventBody::AllSoundOff,
        7 => EventBody::ResetControllers,
        8 => EventBody::MasterVolume { level: a },
        _ => return None,
    };
    Some(Event::new(offset, body))
}

/// Serializa un fixture a texto determinista.
#[must_use]
pub fn encode(fixture: &Fixture) -> String {
    let mut text = String::new();
    text.push_str(&format!("KHDSP {}\n", FIXTURE_VERSION));
    text.push_str(&format!("name {}\n", fixture.name));
    text.push_str(&format!(
        "sample_rate {}\n",
        encode_f64(fixture.sample_rate)
    ));
    text.push_str(&format!("block_size {}\n", fixture.block_size));
    let values = params_values(&fixture.params);
    let params: Vec<String> = values.iter().map(|v| encode_f64(*v)).collect();
    text.push_str(&format!("params {}\n", params.join(" ")));
    text.push_str(&format!("events {}\n", fixture.events.len()));
    for event in &fixture.events {
        text.push_str(&format!("e {}\n", event_line(event)));
    }
    text.push_str(&format!("frames {}\n", fixture.left.len()));
    let left: Vec<String> = fixture.left.iter().map(|v| encode_f64(*v)).collect();
    let right: Vec<String> = fixture.right.iter().map(|v| encode_f64(*v)).collect();
    text.push_str(&format!("left {}\n", left.join(" ")));
    text.push_str(&format!("right {}\n", right.join(" ")));
    text
}

/// Deserializa un fixture. Devuelve `None` si el formato no es válido.
#[must_use]
pub fn decode(text: &str) -> Option<Fixture> {
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next()?.split_whitespace().collect();
    if header.len() != 2 || header[0] != "KHDSP" {
        return None;
    }
    if header[1].parse::<u32>().ok()? != FIXTURE_VERSION {
        return None;
    }

    let mut name = String::new();
    let mut sample_rate = 0.0;
    let mut block_size = 0;
    let mut params = Params::default();
    let mut events = Vec::new();
    let mut left = Vec::new();
    let mut right = Vec::new();

    for line in lines {
        let mut parts = line.splitn(2, ' ');
        let key = parts.next()?;
        let rest = parts.next().unwrap_or("");
        match key {
            "name" => name = rest.to_string(),
            "sample_rate" => sample_rate = decode_f64(rest)?,
            "block_size" => block_size = rest.parse().ok()?,
            "params" => {
                let values: Vec<f64> = rest
                    .split_whitespace()
                    .map(decode_f64)
                    .collect::<Option<Vec<_>>>()?;
                let array: [f64; 19] = values.try_into().ok()?;
                params = params_from_values(&array);
            }
            "events" => {
                let _count: usize = rest.parse().ok()?;
            }
            "e" => events.push(parse_event(rest)?),
            "frames" => {
                let _frames: usize = rest.parse().ok()?;
            }
            "left" => {
                left = rest
                    .split_whitespace()
                    .map(decode_f64)
                    .collect::<Option<Vec<_>>>()?;
            }
            "right" => {
                right = rest
                    .split_whitespace()
                    .map(decode_f64)
                    .collect::<Option<Vec<_>>>()?;
            }
            _ => return None,
        }
    }

    if left.len() != right.len() || name.is_empty() || sample_rate <= 0.0 {
        return None;
    }
    Some(Fixture {
        name,
        sample_rate,
        block_size,
        params,
        events,
        left,
        right,
    })
}
