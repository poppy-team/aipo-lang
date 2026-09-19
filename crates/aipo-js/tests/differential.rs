//! VM↔JS differential suite for the Wave 2 MVP parity slice.
//!
//! For every `docs/conformance/programs/NN_*.aipo` fixture the harness runs the
//! program twice — once on the Rust VM, once on `node` over the emitted JS
//! bundle — and requires VM stdout == JS stdout == committed `.stdout`.
//!
//! Run with:
//!
//! ```bash
//! cargo test -p aipo-js --test differential
//! ```

#![forbid(unsafe_code)]

use aipo_source::{Source, SourceId};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard};

static IO_LOCK: Mutex<()> = Mutex::new(());

fn lock_io() -> MutexGuard<'static, ()> {
    IO_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn conformance_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir.push("docs");
    dir.push("conformance");
    dir
}

fn program_cases() -> Vec<(PathBuf, PathBuf)> {
    let dir = conformance_dir().join("programs");
    let mut cases = Vec::new();
    let entries = std::fs::read_dir(&dir).expect("conformance programs dir");
    for entry in entries {
        let path = entry.expect("dir entry").path();
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

struct Compiled {
    ir: aipo_ir::CoreModule,
    bytecode: aipo_bytecode::BytecodeModule,
}

fn compile(path: &Path) -> Compiled {
    let text = std::fs::read_to_string(path).expect("read fixture");
    let source = Source::new(SourceId::next(), path.display().to_string(), &text);
    let (program, mut diagnostics) = aipo_syntax::parse(&source);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != aipo_diagnostics::Severity::Error),
        "parse errors in {}: {diagnostics:?}",
        path.display()
    );
    let hir = aipo_hir::lower(program);
    let surface = prelude_surface();
    let (_, sema_diagnostics) = aipo_sema::check_with_prelude(&source, &hir, &surface);
    diagnostics.extend(sema_diagnostics);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != aipo_diagnostics::Severity::Error),
        "sema errors in {}: {diagnostics:?}",
        path.display()
    );
    let ir = aipo_ir::lower_to_ir(&hir);
    let bytecode = aipo_bytecode::compile(&ir).expect("bytecode verifies");
    Compiled { ir, bytecode }
}

fn run_vm(compiled: &Compiled) -> String {
    let _guard = lock_io();
    struct BufWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
    impl Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    aipo_stdlib::io::set_output_sink(Some(Box::new(BufWriter(buffer.clone()))));
    let (mut vm, _) = {
        let mut vm = aipo_vm::Vm::new();
        let mut registry = aipo_runtime::NativeRegistry::new();
        aipo_stdlib::register_stdlib(&mut vm, &mut registry);
        (vm, registry)
    };
    for decl in &compiled.bytecode.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    for function in &compiled.bytecode.functions {
        if let Some((type_name, method)) = function.name.split_once('.') {
            vm.register_struct_method(type_name, method, function.entry_ip, function.params);
        }
    }
    vm.run(&compiled.bytecode).expect("vm runs fixture");
    aipo_stdlib::io::set_output_sink(None);
    drop(_guard);
    let bytes = buffer.lock().unwrap_or_else(|e| e.into_inner()).clone();
    String::from_utf8(bytes).expect("vm output is utf-8")
}

fn run_js(path: &Path, compiled: &Compiled) -> String {
    let bundle = aipo_js::emit_js(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("main.aipo"),
        &std::fs::read_to_string(path).expect("read fixture"),
        &compiled.ir,
    );
    let dir = std::env::temp_dir().join(format!(
        "aipo-js-diff-{}",
        path.file_stem().and_then(|n| n.to_str()).unwrap_or("prog")
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join("app.js"), &bundle.app_js).expect("write app.js");
    std::fs::write(dir.join("aipo-runtime.js"), &bundle.runtime_js).expect("write shim");
    std::fs::write(dir.join("app.js.map"), &bundle.source_map).expect("write map");
    let output = Command::new("node")
        .arg("app.js")
        .current_dir(&dir)
        .output()
        .expect("node runs (requires Node >= 20)");
    assert!(
        output.status.success(),
        "node failed for {}: status {:?} stderr: {}",
        path.display(),
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("js output is utf-8")
}

#[test]
fn test_vm_and_js_agree_with_committed_stdout() {
    let cases = program_cases();
    assert!(
        cases.len() >= 20,
        "expected 20 program fixtures, found {}",
        cases.len()
    );
    for (path, stdout_path) in cases {
        let compiled = compile(&path);
        let expected = std::fs::read_to_string(&stdout_path).expect("read snapshot");
        let vm_out = run_vm(&compiled);
        assert_eq!(
            vm_out,
            expected,
            "VM output differs from committed snapshot for {}",
            path.display()
        );
        let js_out = run_js(&path, &compiled);
        assert_eq!(
            js_out,
            expected,
            "JS output differs from committed snapshot for {}",
            path.display()
        );
    }
}

#[test]
fn test_emitted_bundle_is_esm_with_source_map() {
    let dir = conformance_dir().join("programs");
    let path = dir.join("01_hello.aipo");
    let compiled = compile(&path);
    let bundle = aipo_js::emit_js("01_hello.aipo", "io.println(1)\n", &compiled.ir);
    assert!(bundle.app_js.contains("runModule"));
    assert!(bundle.app_js.contains("sourceMappingURL=app.js.map"));
    let map: serde_json::Value =
        serde_json::from_str(&bundle.source_map).expect("source map is JSON");
    assert_eq!(map["version"], 3);
    assert!(bundle.runtime_js.contains("RUNTIME_VERSION"));
    let _ = Write::write_all(&mut std::io::sink(), b"");
}
