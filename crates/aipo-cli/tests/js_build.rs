//! Build parity suite for the Wave 2 JavaScript backend (`P01-G01`).
//!
//! `aipo build` must accept exactly what `aipo run` accepts and reject exactly
//! what `aipo check` rejects: every `diagnostics/` fixture fails with its
//! committed code and emits no files, while runnable programs and the module
//! entry bundle to ESM that `node` executes with the committed stdout.

use std::path::{Path, PathBuf};
use std::process::Command;

const EXIT_SUCCESS: u8 = 0;
const EXIT_LANGUAGE_FAILURE: u8 = 1;

fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/conformance")
}

fn run_build(entry: &Path, out_dir: &Path) -> (u8, String, String) {
    let _ = std::fs::remove_dir_all(out_dir);
    let args = vec![
        "build".to_string(),
        entry.to_string_lossy().into_owned(),
        "--out".to_string(),
        out_dir.to_string_lossy().into_owned(),
    ];
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

fn run_node(dir: &Path) -> (bool, String, String) {
    let output = Command::new("node")
        .arg("app.js")
        .current_dir(dir)
        .output()
        .expect("node runs the bundle (requires Node >= 20)");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("aipo-build-{name}"))
}

fn expected_codes(code_path: &Path) -> Vec<String> {
    std::fs::read_to_string(code_path)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", code_path.display()))
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn test_build_emits_bundle_and_node_matches_snapshot() {
    for name in ["01_hello", "10_integrated", "20_module_scope"] {
        let entry = conformance_dir()
            .join("programs")
            .join(format!("{name}.aipo"));
        let expected = std::fs::read_to_string(
            conformance_dir()
                .join("programs")
                .join(format!("{name}.stdout")),
        )
        .expect("snapshot is readable");
        let dir = temp_dir(name);
        let (code, _, stderr) = run_build(&entry, &dir);
        assert_eq!(code, EXIT_SUCCESS, "build {name} succeeds: {stderr}");
        for file in ["app.js", "aipo-runtime.js", "app.js.map"] {
            assert!(dir.join(file).exists(), "build {name} emits {file}");
        }
        let app = std::fs::read_to_string(dir.join("app.js")).expect("app.js is readable");
        assert!(
            app.contains("sourceMappingURL=app.js.map"),
            "app.js links its map"
        );
        let map = std::fs::read_to_string(dir.join("app.js.map")).expect("map is readable");
        assert!(map.contains("\"version\":3"), "source map is ECMA-426");
        let (ok, stdout, stderr) = run_node(&dir);
        assert!(ok, "node runs {name}: {stderr}");
        assert_eq!(stdout, expected, "node output matches snapshot for {name}");
    }
}

#[test]
fn test_build_rejects_diagnostic_fixtures_with_committed_codes() {
    let dir = conformance_dir().join("diagnostics");
    let mut count = 0;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("diagnostics dir is readable")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("aipo"))
        .collect();
    entries.sort();
    for entry in entries {
        let stem = entry.file_stem().and_then(|n| n.to_str()).unwrap_or("case");
        let codes = expected_codes(&entry.with_extension("code"));
        assert!(!codes.is_empty(), "{stem} has committed codes");
        let out_dir = temp_dir(&format!("diag-{stem}"));
        let (code, _, stderr) = run_build(&entry, &out_dir);
        if code == EXIT_LANGUAGE_FAILURE {
            // Check-time failure (parse/sema): same codes as `check`, no bundle emitted.
            for expected in &codes {
                assert!(
                    stderr.contains(expected),
                    "{stem} reports {expected}:\n{stderr}"
                );
            }
            assert!(
                !out_dir.join("app.js").exists(),
                "{stem} emits no bundle on failure"
            );
        } else {
            // Runtime fault: the bundle builds, and `node` reports the same code.
            assert_eq!(code, EXIT_SUCCESS, "build decides {stem}: {stderr}");
            assert!(out_dir.join("app.js").exists(), "{stem} emits a bundle");
            let (ok, _, js_stderr) = run_node(&out_dir);
            assert!(!ok, "node faults on {stem}");
            for expected in &codes {
                assert!(
                    js_stderr.contains(expected),
                    "node reports {expected} for {stem}:\n{js_stderr}"
                );
            }
        }
        count += 1;
    }
    assert!(
        count >= 19,
        "all diagnostic fixtures covered, found {count}"
    );
}

#[test]
fn test_build_bundles_module_entry_and_rejects_bad_modules() {
    let modules = conformance_dir().join("modules");
    let basic = modules.join("basic").join("entry.aipo");
    let expected =
        std::fs::read_to_string(modules.join("basic").join("entry.stdout")).expect("snapshot");
    let dir = temp_dir("modules-basic");
    let (code, _, stderr) = run_build(&basic, &dir);
    assert_eq!(code, EXIT_SUCCESS, "build bundles modules/basic: {stderr}");
    let (ok, stdout, js_stderr) = run_node(&dir);
    assert!(ok, "node runs bundled entry: {js_stderr}");
    assert_eq!(stdout, expected, "bundled entry matches snapshot");

    for (case, code_file) in [("cycle", "entry.code"), ("missing", "entry.code")] {
        let entry = modules.join(case).join("entry.aipo");
        let codes = expected_codes(&modules.join(case).join(code_file));
        let out_dir = temp_dir(&format!("modules-{case}"));
        let (code, _, stderr) = run_build(&entry, &out_dir);
        assert_eq!(
            code, EXIT_LANGUAGE_FAILURE,
            "build rejects modules/{case}: {stderr}"
        );
        for expected in &codes {
            assert!(
                stderr.contains(expected),
                "modules/{case} reports {expected}:\n{stderr}"
            );
        }
        assert!(
            !out_dir.join("app.js").exists(),
            "modules/{case} emits no bundle on failure"
        );
    }
}
