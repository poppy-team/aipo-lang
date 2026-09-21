//! Source-map conformance beyond `{"version": 3}`.
//!
//! For representative programs the suite asserts the full ECMA-426 shape:
//! version, file, sources, sourcesContent round-trip, per-line VLQ mappings
//! that decode inside the entry source, Unicode/blank-line/EOF behavior, and
//! nested-function programs. A documented limitation is pinned, not hidden:
//! JS-side fault stacks point into the shim (which ships no map), so fault
//! *location* mapping is asserted only as far as the bundle map reaches.

#![forbid(unsafe_code)]

use aipo_testkit::{corpus, js, pipeline};

fn bundle_for(name: &str, source: &str) -> (String, serde_json::Value) {
    let (_, ir) =
        pipeline::lower_to_ir(name, source).unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"));
    let bundle = aipo_js::emit_js(name, source, &ir);
    let map: serde_json::Value = serde_json::from_str(&bundle.source_map).expect("map is JSON");
    (bundle.app_js, map)
}

fn assert_map_shape(name: &str, source: &str, app_js: &str, map: &serde_json::Value) {
    assert_eq!(map["version"], 3, "{name}: version");
    assert_eq!(map["file"], "app.js", "{name}: file");
    assert_eq!(map["sources"][0].as_str(), Some(name), "{name}: sources");
    assert_eq!(
        map["sourcesContent"][0].as_str(),
        Some(source),
        "{name}: sourcesContent round-trips the entry"
    );
    assert!(
        map["names"]
            .as_array()
            .is_some_and(|names| names.is_empty())
    );
    let mappings = map["mappings"].as_str().expect("mappings is a string");
    let app_lines = app_js.lines().count();
    let src_lines = source.lines().count().max(1);
    let segments = js::decode_mappings(mappings);
    assert_eq!(
        mappings.split(';').count(),
        app_lines,
        "{name}: one mapping row per generated line"
    );
    for segment in &segments {
        assert_eq!(segment.source, 0, "{name}: single source");
        assert!(
            (segment.source_line as usize) < src_lines,
            "{name}: generated line {} maps to source line {} inside {}",
            segment.generated_line,
            segment.source_line,
            src_lines
        );
        assert_eq!(segment.source_column, 0, "{name}: line granularity");
    }
    assert!(!segments.is_empty(), "{name}: map is not vacuous");
}

#[test]
fn test_maps_cover_representative_programs() {
    for entry in [
        "01_hello.aipo",
        "03_control_flow.aipo",
        "05_structs_and_impl.aipo",
        "06_closures.aipo",
        "10_integrated.aipo",
        "19_local_functions.aipo",
    ] {
        let path = corpus::workspace_dir()
            .join("docs/conformance/programs")
            .join(entry);
        let source = std::fs::read_to_string(&path).expect("fixture is readable");
        let (app_js, map) = bundle_for(entry, &source);
        assert_map_shape(entry, &source, &app_js, &map);
    }
}

#[test]
fn test_map_handles_unicode_blank_lines_and_eof() {
    let source = "io.println(\"caf\u{e9} \u{4e2d} \u{1f389}\")\n\n\nio.println(1)";
    let (app_js, map) = bundle_for("unicode.aipo", source);
    assert_map_shape("unicode.aipo", source, &app_js, &map);
    // sourcesContent preserves the exact bytes, including multibyte characters.
    assert!(
        map["sourcesContent"][0]
            .as_str()
            .is_some_and(|content| content.contains('\u{1f389}'))
    );
    // No trailing newline: the last generated row still maps inside the source.
    let plain = "io.println(1)";
    let (app_js, map) = bundle_for("noeof.aipo", plain);
    assert_map_shape("noeof.aipo", plain, &app_js, &map);
}

#[test]
fn test_module_entry_map_names_the_entry() {
    let path = corpus::workspace_dir().join("docs/conformance/modules/basic/entry.aipo");
    let source = std::fs::read_to_string(&path).expect("entry is readable");
    // Module resolution happens before emission (CLI `build` path); the map
    // names the entry file the bundle was built from.
    let (app_js, map) = bundle_for("entry.aipo", &source);
    assert_map_shape("entry.aipo", &source, &app_js, &map);
}

#[test]
fn test_faulting_bundle_reports_code_and_keeps_valid_map() {
    // A faulting program still emits a valid map; the observable fault channel
    // is the `error: [CODE]` line on stderr (JS stacks point into the shim,
    // which ships no map — documented limitation, asserted here as behavior).
    let source = "io.println(1 / 0)\n";
    let (app_js, map) = bundle_for("divzero.aipo", source);
    assert_map_shape("divzero.aipo", source, &app_js, &map);
    let (_, ir) = pipeline::lower_to_ir("divzero.aipo", source).unwrap();
    let bundle = js::emit_bundle("divzero.aipo", source, &ir, "sourcemap-fault");
    let (code, _stdout, stderr) = js::run_node(&bundle.dir);
    let _ = std::fs::remove_dir_all(&bundle.dir);
    assert_ne!(code, Some(0));
    assert!(stderr.contains("AIPO_RT_DIV_ZERO"));
}
