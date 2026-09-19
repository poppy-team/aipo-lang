//! Golden corpus for the canonical formatter.
//!
//! Every `*.input.aipo` under `docs/conformance/formatting` is formatted and compared with
//! the committed `*.expected.aipo`, then formatted again to prove idempotence. The corpus is
//! the artifact required by slice S10 (CLI + formatter): it pins the canonical style and
//! fails loudly if a formatting rule drifts.
//!
//! Regenerate the expectations with `AIPO_UPDATE_GOLDEN=1 cargo test -p aipo-formatter`.

use aipo_formatter::format_text;
use std::path::{Path, PathBuf};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/conformance/formatting")
}

fn corpus_inputs() -> Vec<PathBuf> {
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(corpus_dir())
        .expect("formatting corpus directory exists")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.to_string_lossy().ends_with(".input.aipo"))
        .collect();
    inputs.sort();
    assert!(
        !inputs.is_empty(),
        "formatting corpus must contain at least one .input.aipo fixture"
    );
    inputs
}

fn expected_path(input: &Path) -> PathBuf {
    PathBuf::from(
        input
            .to_string_lossy()
            .replace(".input.aipo", ".expected.aipo"),
    )
}

#[test]
fn test_golden_corpus_matches_expectations() {
    let update = std::env::var("AIPO_UPDATE_GOLDEN").is_ok();
    for input in corpus_inputs() {
        let source = std::fs::read_to_string(&input).expect("fixture is readable");
        let name = input.to_string_lossy().to_string();
        let formatted = format_text(&name, &source).expect("fixture must tokenize");

        let expected_file = expected_path(&input);
        if update || !expected_file.exists() {
            std::fs::write(&expected_file, &formatted).expect("expectation is writable");
        }

        let expected = std::fs::read_to_string(&expected_file).expect("expectation is readable");
        assert_eq!(
            formatted,
            expected,
            "{} is not formatted canonically",
            input.display()
        );
    }
}

#[test]
fn test_formatting_is_idempotent_over_corpus() {
    for input in corpus_inputs() {
        let source = std::fs::read_to_string(&input).expect("fixture is readable");
        let name = input.to_string_lossy().to_string();
        let once = format_text(&name, &source).expect("fixture must tokenize");
        let twice = format_text(&name, &once).expect("formatted output must tokenize");
        assert_eq!(
            once,
            twice,
            "formatting is not idempotent for {}",
            input.display()
        );
    }
}

#[test]
fn test_expected_output_is_stable_under_reformatting() {
    for input in corpus_inputs() {
        let expected_file = expected_path(&input);
        let expected = std::fs::read_to_string(&expected_file).expect("expectation is readable");
        let name = expected_file.to_string_lossy().to_string();
        let again = format_text(&name, &expected).expect("expectation must tokenize");
        assert_eq!(
            again,
            expected,
            "{} is already formatted, so reformatting must be a no-op",
            expected_file.display()
        );
    }
}

#[test]
fn test_every_fixture_is_valid_aipo_source() {
    // A formatting fixture that does not even tokenize would silently pass the golden
    // comparison, so each input is required to parse as well.
    for input in corpus_inputs() {
        let source = std::fs::read_to_string(&input).expect("fixture is readable");
        assert!(
            format_text(&input.to_string_lossy(), &source).is_ok(),
            "{} must be valid Aipo source",
            input.display()
        );
    }
}
