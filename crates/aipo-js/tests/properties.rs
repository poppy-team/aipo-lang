//! Property-style tests for the `aipo-js` backend (`P01-G01`, Table 1 item 11).
//!
//! The corpus differential suite proves example parity; these tests prove
//! backend *invariants* that hold for every program:
//!
//! - emission is deterministic (same IR emits byte-identical bundles);
//! - every bundle is coherent (ESM wrapper links a valid ECMA-426 map whose
//!   mappings decode and stay inside the entry source; the three artifacts
//!   agree on `RUNTIME_VERSION`);
//! - the shim's pure value/stdlib layer satisfies its unit properties
//!   (`runtime/selftest.mjs` under `node`).

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

fn workspace_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir
}

fn program_sources() -> Vec<PathBuf> {
    let dir = workspace_dir().join("docs/conformance/programs");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("programs dir is readable")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("aipo"))
        .collect();
    paths.sort();
    assert!(paths.len() >= 21, "corpus programs present");
    paths
}

fn compile_to_ir(path: &Path) -> (String, aipo_ir::CoreModule) {
    let text = std::fs::read_to_string(path).expect("fixture is readable");
    let source = aipo_source::Source::new(
        aipo_source::SourceId::next(),
        path.display().to_string(),
        &text,
    );
    let (program, diagnostics) = aipo_syntax::parse(&source);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != aipo_diagnostics::Severity::Error),
        "fixture parses: {}",
        path.display()
    );
    let hir = aipo_hir::lower(program);
    let surface = prelude_surface();
    let (_, sema_diagnostics) = aipo_sema::check_with_prelude(&source, &hir, &surface);
    assert!(
        sema_diagnostics
            .iter()
            .all(|d| d.severity != aipo_diagnostics::Severity::Error),
        "fixture checks: {}",
        path.display()
    );
    (text, aipo_ir::lower_to_ir(&hir))
}

/// Semantic surface derived from the registered standard library.
///
/// Deriving it keeps these properties honest as the stdlib grows: a module added to the
/// runtime is visible here without a second hand-maintained list to drift.
fn prelude_surface() -> aipo_sema::PreludeSurface {
    let mut vm = aipo_vm::Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    let mut surface = aipo_sema::PreludeSurface::fundamental();
    for name in vm.globals.keys() {
        match registry.get(None, name) {
            Some(meta) => surface.add_function(name, meta.arity, meta.arity),
            None => surface.add_variable(name),
        }
    }
    surface
}

/// Mapping coherence is asserted through `aipo_testkit::js::decode_mappings`.

#[test]
fn test_emission_is_deterministic_over_the_corpus() {
    for path in program_sources() {
        let (text, ir) = compile_to_ir(&path);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("main.aipo");
        let first = aipo_js::emit_js(name, &text, &ir);
        let second = aipo_js::emit_js(name, &text, &ir);
        assert_eq!(
            first.app_js, second.app_js,
            "app.js deterministic for {name}"
        );
        assert_eq!(
            first.source_map, second.source_map,
            "source map deterministic for {name}"
        );
    }
}

#[test]
fn test_bundles_are_coherent_over_the_corpus() {
    for path in program_sources() {
        let (text, ir) = compile_to_ir(&path);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("main.aipo");
        let bundle = aipo_js::emit_js(name, &text, &ir);

        assert!(
            bundle.app_js.contains("sourceMappingURL=app.js.map"),
            "{name} links its map"
        );
        assert!(
            bundle.app_js.contains(aipo_js::RUNTIME_VERSION),
            "{name} records the shim version"
        );
        assert!(
            bundle.runtime_js.contains(aipo_js::RUNTIME_VERSION),
            "shim carries its version"
        );

        let map: serde_json::Value = serde_json::from_str(&bundle.source_map).expect("map is JSON");
        assert_eq!(map["version"], 3, "{name} map is ECMA-426");
        assert_eq!(map["sources"][0].as_str(), Some(name));
        let mappings = map["mappings"].as_str().expect("mappings is a string");
        let app_lines = bundle.app_js.lines().count();
        let src_lines = text.lines().count().max(1);
        for segment in aipo_testkit::js::decode_mappings(mappings) {
            assert_eq!(segment.source, 0, "{name} maps the entry source");
            assert!(
                (segment.source_line as usize) < src_lines,
                "{name} line {} maps inside the entry source",
                segment.generated_line
            );
            assert_eq!(app_lines, mappings.split(';').count());
        }
    }
}

#[test]
fn test_shim_selftest_passes_under_node() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new("node")
        .arg("runtime/selftest.mjs")
        .current_dir(&manifest)
        .output()
        .expect("node runs the shim self-test (requires Node >= 20)");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "shim self-test failed:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !stderr.contains("AssertionError"),
        "shim self-test assertion failed:\n{stderr}"
    );
}
