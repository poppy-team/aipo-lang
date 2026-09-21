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

/// The deterministic clock profile this suite runs both backends under.
///
/// `time.now`/`time.monotonic` are host capabilities, so a parity comparison is only
/// meaningful when the two backends read the *same* source: the VM installs
/// [`DeterministicClock`] and the JavaScript side gets its twin as a preload module (see
/// [`write_clock_profile`]). Wall time is a fixed instant and the monotonic origin is zero, so
/// a fixture can assert the clock contracts and still produce a stable, comparable result.
const PROFILE_WALL_SECONDS: f64 = 1_000_000.0;
const PROFILE_MONOTONIC_SECONDS: f64 = 0.0;

/// The VM-side half of the deterministic clock profile.
struct DeterministicClock;

impl aipo_stdlib::time::ClockSource for DeterministicClock {
    fn wall_seconds(&self) -> f64 {
        PROFILE_WALL_SECONDS
    }

    fn monotonic_seconds(&self) -> f64 {
        PROFILE_MONOTONIC_SECONDS
    }
}

/// Writes the JavaScript half of the deterministic clock profile, returning the preload file
/// name to hand to `node --import`.
///
/// The emitted entry installs the system clock only when no source is present, so loading this
/// module first is what makes a replay replay. That is the same mechanism a host uses, not a
/// hook the runtime reserves for tests.
fn write_clock_profile(dir: &Path) -> &'static str {
    let source = format!(
        "// Deterministic clock profile for the VM/JS differential suite.\n\
         globalThis.__aipoClock = {{\n\
         \x20 wallSeconds: () => {PROFILE_WALL_SECONDS},\n\
         \x20 monotonicSeconds: () => {PROFILE_MONOTONIC_SECONDS},\n\
         }};\n"
    );
    std::fs::write(dir.join("clock-profile.mjs"), source).expect("write clock profile");
    "clock-profile.mjs"
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
    // The deterministic profile: the same clock the preload installs on the JavaScript side.
    aipo_stdlib::time::install_clock(Box::new(DeterministicClock));
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
            vm.register_struct_method(
                type_name,
                method,
                function.entry_ip,
                function.params,
                function.is_async,
            );
        }
    }
    vm.run(&compiled.bytecode).expect("vm runs fixture");
    aipo_stdlib::io::set_output_sink(None);
    aipo_stdlib::time::revoke_clock();
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
    let profile = write_clock_profile(&dir);
    let output = Command::new("node")
        .arg("--import")
        .arg(format!("./{profile}"))
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

fn compile_code(name: &str, text: &str) -> Compiled {
    let source = Source::new(SourceId::next(), name.to_string(), text);
    let (program, mut diagnostics) = aipo_syntax::parse(&source);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != aipo_diagnostics::Severity::Error),
        "parse errors in {name}: {diagnostics:?}"
    );
    let hir = aipo_hir::lower(program);
    let surface = prelude_surface();
    let (_, sema_diagnostics) = aipo_sema::check_with_prelude(&source, &hir, &surface);
    diagnostics.extend(sema_diagnostics);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != aipo_diagnostics::Severity::Error),
        "sema errors in {name}: {diagnostics:?}"
    );
    let ir = aipo_ir::lower_to_ir(&hir);
    let bytecode = aipo_bytecode::compile(&ir).expect("bytecode verifies");
    Compiled { ir, bytecode }
}

fn run_js_code(name: &str, text: &str, compiled: &Compiled) -> String {
    let bundle = aipo_js::emit_js(name, text, &compiled.ir);
    let dir = std::env::temp_dir().join(format!("aipo-js-diff-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join("app.js"), &bundle.app_js).expect("write app.js");
    std::fs::write(dir.join("aipo-runtime.js"), &bundle.runtime_js).expect("write shim");
    std::fs::write(dir.join("app.js.map"), &bundle.source_map).expect("write map");
    let profile = write_clock_profile(&dir);
    let output = Command::new("node")
        .arg("--import")
        .arg(format!("./{profile}"))
        .arg("app.js")
        .current_dir(&dir)
        .output()
        .expect("node runs (requires Node >= 20)");
    assert!(
        output.status.success(),
        "node failed for {name}: status {:?} stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("node stdout is utf-8")
}

#[test]
fn test_wave3_differential_parity() {
    let code = r#"
var s = Set([1, 2, 2, 3])
s.add(4)
io.println(s.has(2))
s.remove(2)
io.println(s.has(2))
io.println(s.len())
io.println(s)

var b = Bytes(4)
b.write_i16(0, -42)
b.write_u16(2, 60000)
io.println(b.read_i16(0))
io.println(b.read_u16(2))
var text = "hello"
var dec = text.encode().decode()
io.println(dec)

var d1 = Duration(1.5)
var d2 = Duration(2.5)
io.println(d1 + d2)
io.println(d2 - d1)
io.println(d1 < d2)
io.println(d1.total_seconds())

var l = [1, 2, 3]
io.println(l.lazy())
"#;

    let compiled = compile_code("wave3_diff.aipo", code);
    let vm_out = run_vm(&compiled);
    let js_out = run_js_code("wave3_diff.aipo", code, &compiled);
    assert_eq!(
        vm_out, js_out,
        "VM and JS outputs must be identical for Wave 3 features"
    );
}

#[test]
fn test_wave3_async_spawn_await_differential() {
    let code = r#"
fn worker(x)
    return x * 2
end

var t = task.spawn(worker, [21])
var res = await t
io.println(res)
"#;
    let compiled = compile_code("wave3_async_spawn.aipo", code);
    let vm_out = run_vm(&compiled);
    let js_out = run_js_code("wave3_async_spawn.aipo", code, &compiled);
    assert_eq!(vm_out, js_out);
    assert_eq!(vm_out, "42\n");
}

#[test]
fn test_wave3_async_all_and_race_differential() {
    let code = r#"
fn f1()
    return 10
end
fn f2()
    return 20
end

var t1 = task.spawn(f1, [])
var t2 = task.spawn(f2, [])
var all_res = task.all([t1, t2])
io.println(all_res)

var race_res = task.race([task.spawn(f1, []), task.spawn(f2, [])])
io.println(race_res)
"#;
    let compiled = compile_code("wave3_async_all_race.aipo", code);
    let vm_out = run_vm(&compiled);
    let js_out = run_js_code("wave3_async_all_race.aipo", code, &compiled);
    assert_eq!(vm_out, js_out);
}

#[test]
fn test_wave3_async_group_differential() {
    let code = r#"
fn mult(val)
    return val * 3
end

var g = task.group()
var t1 = g.spawn(mult, [4])
var t2 = g.spawn(mult, [5])
var res = g.wait()
io.println(res)
"#;
    let compiled = compile_code("wave3_async_group.aipo", code);
    let vm_out = run_vm(&compiled);
    let js_out = run_js_code("wave3_async_group.aipo", code, &compiled);
    assert_eq!(vm_out, js_out);
    assert_eq!(vm_out, "[12, 15]\n");
}

#[test]
fn test_wave3_async_sleep_and_timeout_differential() {
    let code = r#"
fn worker()
    task.sleep(1)
    return 99
end

var t = task.spawn(worker, [])
var res = task.timeout(t, 2)
io.println(res)

fn slow_worker()
    task.sleep(5)
    return 100
end

var t2 = task.spawn(slow_worker, [])
var timeout_res = task.timeout(t2, 1) or_else "timed_out"
io.println(timeout_res)
"#;
    let compiled = compile_code("wave3_async_sleep_timeout.aipo", code);
    let vm_out = run_vm(&compiled);
    let js_out = run_js_code("wave3_async_sleep_timeout.aipo", code, &compiled);
    assert_eq!(vm_out, js_out);
    assert_eq!(vm_out, "99\ntimed_out\n");
}

#[test]
fn test_wave3_async_cancel_differential() {
    let code = r#"
fn worker()
    task.sleep(10)
    return 1
end

var t = task.spawn(worker, [])
task.cancel(t)
io.println("cancelled ok")
"#;
    let compiled = compile_code("wave3_async_cancel.aipo", code);
    let vm_out = run_vm(&compiled);
    let js_out = run_js_code("wave3_async_cancel.aipo", code, &compiled);
    assert_eq!(vm_out, js_out);
    assert_eq!(vm_out, "cancelled ok\n");
}

#[test]
fn test_time_clock_differential_under_the_deterministic_profile() {
    // The clock is a host capability, so parity means the two backends read the same source:
    // the readings themselves must match, not merely the booleans derived from them.
    let code = r#"
io.println(time.now().total_seconds())
io.println(time.monotonic().total_seconds())
io.println(time.monotonic().total_seconds() >= 0.0)
"#;
    let compiled = compile_code("wave4_time_diff.aipo", code);
    let vm_out = run_vm(&compiled);
    let js_out = run_js_code("wave4_time_diff.aipo", code, &compiled);
    assert_eq!(vm_out, js_out, "VM and JS must read the same clock profile");
    // Aipo prints a whole Float with its fraction (`1000000.0`), so the profile above is what
    // makes both readings exact and comparable on both backends.
    assert_eq!(
        vm_out, "1000000.0\n0.0\ntrue\n",
        "the profile decides both readings"
    );
}

#[test]
fn test_denied_clock_faults_identically_on_both_backends() {
    // A host that grants nothing must not get a faked clock on either backend, and the fault
    // must carry the canon code rather than a silent zero.
    let code = "io.println(time.monotonic().total_seconds())\n";
    let compiled = compile_code("wave4_time_denied.aipo", code);

    // The clock is process-global, so this test must exclude the ones that install a source.
    let _guard = lock_io();
    let mut vm = aipo_vm::Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    aipo_stdlib::time::revoke_clock();
    let vm_error = vm.run(&compiled.bytecode).expect_err("denied on the VM");
    assert_eq!(
        vm_error.diagnostic_code(),
        aipo_diagnostics::DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
    );

    let bundle = aipo_js::emit_js("wave4_time_denied.aipo", code, &compiled.ir);
    let dir = std::env::temp_dir().join("aipo-js-diff-wave4_time_denied");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join("app.js"), &bundle.app_js).expect("write app.js");
    std::fs::write(dir.join("aipo-runtime.js"), &bundle.runtime_js).expect("write shim");
    std::fs::write(dir.join("app.js.map"), &bundle.source_map).expect("write map");
    // Deny the capability: the preload removes whatever the entry installed.
    std::fs::write(
        dir.join("deny-clock.mjs"),
        "globalThis.__aipoClock = null;\n",
    )
    .expect("write deny profile");
    let output = Command::new("node")
        .arg("--import")
        .arg("./deny-clock.mjs")
        .arg("app.js")
        .current_dir(&dir)
        .output()
        .expect("node runs (requires Node >= 20)");
    assert!(
        !output.status.success(),
        "a denied clock must fail the JS run too"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("AIPO_RT_CAPABILITY_DENIED"),
        "JS denial must report the canon code, got: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
