//! Harness del corpus de regresión congelado (`docs/TESTING.md` §4).
//!
//! No reescribe los fixtures: solo verifica que el render actual coincide con
//! el guardado. Para un cambio de sonido intencional se crea `contract/v2/`.

use std::path::{Path, PathBuf};

use khodes_dsp::contract_format::{Fixture, decode};
use khodes_dsp::{Engine, Event};

const TOLERANCE: f64 = 1e-7;

fn corpus_dir() -> PathBuf {
    for candidate in ["contract/v1", "../contract/v1"] {
        let path = Path::new(candidate);
        if path.is_dir() {
            return path.to_path_buf();
        }
    }
    panic!("no se encontró contract/v1");
}

fn render(fixture: &Fixture) -> (Vec<f64>, Vec<f64>) {
    let mut engine = Engine::default();
    engine.prepare(fixture.sample_rate, fixture.block_size);
    let frames = fixture.left.len();
    let mut left = vec![0.0; frames];
    let mut right = vec![0.0; frames];

    let mut pos = 0;
    while pos < frames {
        let end = (pos + fixture.block_size).min(frames);
        let block_events: Vec<Event> = fixture
            .events
            .iter()
            .filter(|e| e.sample_offset >= pos && e.sample_offset < end)
            .map(|e| Event::new(e.sample_offset - pos, e.body))
            .collect();
        engine.process_stereo(
            &mut left[pos..end],
            &mut right[pos..end],
            &block_events,
            &fixture.params,
        );
        pos = end;
    }
    (left, right)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE + TOLERANCE * a.abs().max(b.abs())
}

#[test]
fn frozen_corpus_v1_matches_current_render() {
    let dir = corpus_dir();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("leer contract/v1")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "khdsp"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "el corpus no tiene fixtures");

    for path in paths {
        let text = std::fs::read_to_string(&path).expect("leer fixture");
        let fixture =
            decode(&text).unwrap_or_else(|| panic!("fixture inválido: {}", path.display()));
        let (left, right) = render(&fixture);

        assert_eq!(left.len(), fixture.left.len(), "{}: frames L", fixture.name);
        assert_eq!(
            right.len(),
            fixture.right.len(),
            "{}: frames R",
            fixture.name
        );

        for (i, (&got, &want)) in left.iter().zip(fixture.left.iter()).enumerate() {
            assert!(
                close(got, want),
                "{}: L[{}] = {:016x}, esperado {:016x}",
                fixture.name,
                i,
                got.to_bits(),
                want.to_bits()
            );
        }
        for (i, (&got, &want)) in right.iter().zip(fixture.right.iter()).enumerate() {
            assert!(
                close(got, want),
                "{}: R[{}] = {:016x}, esperado {:016x}",
                fixture.name,
                i,
                got.to_bits(),
                want.to_bits()
            );
        }
    }
}
