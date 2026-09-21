//! Metamorphic equivalence suite: semantics-preserving rewrites must not
//! change observable behavior on either backend.
//!
//! Each transform documents its own equivalence argument in `aipo-testkit::meta`.
//! The oracle runs original and rewritten sources on the VM and on Node and
//! requires all four executions to agree (stdout and exit code).

#![forbid(unsafe_code)]

use aipo_testkit::{corpus, js, meta, pipeline, rng::Rng, smith};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn vm_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn observable_vm(source: &str) -> Result<String, String> {
    pipeline::run_text("meta.aipo", source).map_err(|failure| format!("{failure:?}"))
}

fn observable_js(source: &str, tag: &str) -> (Option<i32>, String) {
    let (ir_text, ir) = pipeline::lower_to_ir("meta.aipo", source)
        .unwrap_or_else(|diagnostics| panic!("valid source failed lowering: {diagnostics:?}"));
    let _ = ir_text;
    let bundle = js::emit_bundle("meta.aipo", source, &ir, tag);
    let (code, stdout, stderr) = js::run_node(&bundle.dir);
    let _ = std::fs::remove_dir_all(&bundle.dir);
    assert!(
        stderr.is_empty() || code != Some(0),
        "node stderr on success: {stderr}"
    );
    (code, stdout)
}

fn assert_metamorphic(label: &str, original: &str, rewritten: &str) {
    // Caller holds `vm_lock` for the whole test: the stdlib output sink is
    // process-global and the guard is not reentrant, so locking here would
    // deadlock nested calls and skipping it contaminates parallel tests.
    assert_metamorphic_inner(label, original, rewritten);
}

fn assert_metamorphic_inner(label: &str, original: &str, rewritten: &str) {
    let vm_original = observable_vm(original);
    let vm_rewritten = observable_vm(rewritten);
    assert_eq!(
        vm_original.is_ok(),
        vm_rewritten.is_ok(),
        "{label}: VM exit agrees before/after rewrite"
    );
    if let (Ok(first), Ok(second)) = (&vm_original, &vm_rewritten) {
        assert_eq!(first, second, "{label}: VM stdout identical after rewrite");
    }
    let tag = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let (code_original, js_original) = observable_js(original, &format!("{tag}-orig"));
    let (code_rewritten, js_rewritten) = observable_js(rewritten, &format!("{tag}-rw"));
    assert_eq!(
        code_original, code_rewritten,
        "{label}: JS exit agrees before/after rewrite"
    );
    assert_eq!(
        js_original, js_rewritten,
        "{label}: JS stdout identical after rewrite"
    );
    match (vm_original, code_original) {
        (Ok(vm_stdout), Some(0)) => assert_eq!(
            vm_stdout, js_original,
            "{label}: VM and JS agree on the original"
        ),
        (Err(_), code) => assert_ne!(code, Some(0), "{label}: JS fails when the VM fails"),
        (Ok(_), code) => panic!("{label}: VM ok but node exit {code:?}"),
    }
}

fn corpus_source(name: &str) -> String {
    std::fs::read_to_string(
        corpus::workspace_dir()
            .join("docs/conformance/programs")
            .join(format!("{name}.aipo")),
    )
    .expect("fixture is readable")
}

#[test]
fn test_comments_and_blanks_preserve_behavior() {
    let _guard = vm_lock();
    let mut rng = Rng::new(20260920);
    for name in [
        "01_hello",
        "03_control_flow",
        "05_structs_and_impl",
        "10_integrated",
    ] {
        let original = corpus_source(name);
        let rewritten = meta::insert_comments_blanks(&mut rng, &original);
        assert_metamorphic(&format!("comments/{name}"), &original, &rewritten);
    }
}

#[test]
fn test_crlf_preserves_behavior() {
    let _guard = vm_lock();
    for name in ["02_recursion", "04_collections", "20_module_scope"] {
        let original = corpus_source(name);
        let rewritten = meta::to_crlf(&original);
        assert_metamorphic(&format!("crlf/{name}"), &original, &rewritten);
    }
}

#[test]
fn test_binding_rename_preserves_behavior() {
    let _guard = vm_lock();
    // `20_module_scope` prints its binding name inside an f-string, so the
    // renamed program legitimately prints different text: here the oracle is
    // VM/JS agreement on the renamed source plus a clean exit, not equality
    // with the original.
    let original = corpus_source("20_module_scope");
    let renamed = meta::rename_binding(
        &meta::rename_binding(&original, "total", "sum_total"),
        "scale",
        "factor",
    );
    let vm_renamed = observable_vm(&renamed);
    let (code, js_renamed) = observable_js(&renamed, "rename-20");
    match (vm_renamed, code) {
        (Ok(vm_stdout), Some(0)) => assert_eq!(vm_stdout, js_renamed),
        (Err(_), other) => assert_ne!(other, Some(0)),
        (Ok(_), other) => panic!("VM ok but node exit {other:?}"),
    }
    assert!(
        js_renamed.contains("sum_total: 5"),
        "rename took effect: {js_renamed}"
    );

    let config = smith::Config::default();
    for seed in [3u64, 11, 77] {
        let generated = smith::generate(seed, &config);
        // `v0` exists in every generated program (first binding is always `v0`).
        let renamed = meta::rename_binding(&generated.source, "v0", "w0");
        assert_metamorphic(&format!("rename/smith-{seed}"), &generated.source, &renamed);
    }
}

#[test]
fn test_nfc_spellings_agree_on_both_backends() {
    let _guard = vm_lock();
    for (composed, decomposed) in meta::nfc_equivalents() {
        let left = format!("io.println(\"{composed}\")\nio.println(len(\"{composed}\"))\n");
        let right = format!("io.println(\"{decomposed}\")\nio.println(len(\"{decomposed}\"))\n");
        assert_metamorphic("nfc/equivalents", &left, &right);
    }
}
