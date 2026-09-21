//! Corpus discovery: the conformance fixtures and examples as test inputs.

use std::path::{Path, PathBuf};

/// Workspace root derived from this crate's manifest location.
#[must_use]
pub fn workspace_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir
}

/// All `*.aipo` files under `docs/conformance/programs` with committed `.stdout`.
#[must_use]
pub fn runnable_programs() -> Vec<(PathBuf, PathBuf)> {
    paired_fixtures(&workspace_dir().join("docs/conformance/programs"), "stdout")
}

/// All `*.aipo` files under `docs/conformance/diagnostics` with committed `.code`.
#[must_use]
pub fn diagnostic_fixtures() -> Vec<(PathBuf, PathBuf)> {
    paired_fixtures(
        &workspace_dir().join("docs/conformance/diagnostics"),
        "code",
    )
}

/// Runnable `examples/*.aipo` (top level) with committed `.stdout`.
#[must_use]
pub fn runnable_examples() -> Vec<(PathBuf, PathBuf)> {
    let dir = workspace_dir().join("examples");
    let mut cases = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return cases;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("aipo") {
            let stdout = path.with_extension("stdout");
            if stdout.exists() {
                cases.push((path, stdout));
            }
        }
    }
    cases.sort();
    cases
}

/// Source texts of every runnable program plus every diagnostic fixture.
#[must_use]
pub fn seed_sources() -> Vec<String> {
    let mut sources = Vec::new();
    for (path, _) in runnable_programs().into_iter().chain(diagnostic_fixtures()) {
        if let Ok(text) = std::fs::read_to_string(&path) {
            sources.push(text);
        }
    }
    sources
}

fn paired_fixtures(dir: &Path, pair_extension: &str) -> Vec<(PathBuf, PathBuf)> {
    let mut cases = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return cases;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("aipo") {
            let pair = path.with_extension(pair_extension);
            if pair.exists() {
                cases.push((path, pair));
            }
        }
    }
    cases.sort();
    cases
}
