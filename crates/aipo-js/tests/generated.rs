//! Generated-program differential suite: VM == JS on AipoSmith programs.
//!
//! The hand-authored corpus proves parity on curated examples; this suite proves
//! it on dozens of machine-generated valid programs. Every case is reproducible
//! from the printed `(seed, config)` pair, and only minimized, interesting
//! failures would ever be promoted to `docs/conformance/` (regression policy).

#![forbid(unsafe_code)]

use aipo_testkit::{corpus, js, pipeline, smith};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn vm_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn assert_parity(seed: u64, source: &str) {
    let _guard = vm_lock();
    let vm_outcome = pipeline::run_text("smith.aipo", source);
    let (ir_text, ir) = pipeline::lower_to_ir("smith.aipo", source)
        .unwrap_or_else(|diagnostics| panic!("seed {seed} failed lowering: {diagnostics:?}"));
    let _ = ir_text;
    let bundle = js::emit_bundle("smith.aipo", source, &ir, &format!("smith-{seed}"));
    let (code, js_stdout, js_stderr) = js::run_node(&bundle.dir);
    let _ = std::fs::remove_dir_all(&bundle.dir);
    match vm_outcome {
        Ok(vm_stdout) => {
            assert_eq!(
                code,
                Some(0),
                "seed {seed}: node exits 0 when the VM succeeds:\n{js_stderr}\nsource:\n{source}"
            );
            assert_eq!(
                js_stdout, vm_stdout,
                "seed {seed}: JS stdout == VM stdout\nsource:\n{source}"
            );
        }
        Err(_) => {
            assert_ne!(
                code,
                Some(0),
                "seed {seed}: node fails when the VM fails:\n{js_stderr}\nsource:\n{source}"
            );
        }
    }
    let _ = corpus::workspace_dir;
}

#[test]
fn test_generated_programs_agree_vm_and_js() {
    let config = smith::Config::default();
    for seed in 0..60u64 {
        let generated = smith::generate(seed, &config);
        assert_parity(seed, &generated.source);
    }
}

#[test]
fn test_generated_programs_agree_vm_and_js_large() {
    let config = smith::Config {
        max_stmts: 40,
        max_fns: 5,
        max_depth: 4,
        max_list_len: 8,
    };
    for seed in 1000..1020u64 {
        let generated = smith::generate(seed, &config);
        assert_parity(seed, &generated.source);
    }
}
