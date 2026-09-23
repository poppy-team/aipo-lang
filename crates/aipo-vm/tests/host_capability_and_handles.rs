//! Host ABI enforcement at the VM pipeline level (Wave 4, `P03-G01`).
//!
//! Every test here drives the real pipeline (parse → HIR → IR → bytecode → VM) rather than
//! poking at a helper, because the claim being made is about the integration.

use std::sync::Mutex;

use aipo_bytecode::BytecodeModule;
use aipo_bytecode::compile;
use aipo_hir::lower;
use aipo_host::HostValue;
use aipo_ir::lower_to_ir;
use aipo_runtime::NativeRegistry;
use aipo_source::{Source, SourceId};
use aipo_stdlib::register_stdlib;
use aipo_stdlib::time::{ClockSource, install_clock, revoke_clock};
use aipo_syntax::parse;
use aipo_vm::{Value, Vm};

/// A controllable source for testing clock capabilities.
struct FixedClock {
    wall: f64,
    monotonic: f64,
}

impl ClockSource for FixedClock {
    fn wall_seconds(&self) -> f64 {
        self.wall
    }

    fn monotonic_seconds(&self) -> f64 {
        self.monotonic
    }
}

/// Serializes the tests: the clock service is process-global by design.
static CLOCK_LOCK: Mutex<()> = Mutex::new(());

fn compile_program(code: &str) -> BytecodeModule {
    let src = Source::new(SourceId::next(), "host_abi.aipo", code);
    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {parse_diags:?}");
    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    compile(&ir).expect("program compiles")
}

fn run_code(code: &str) -> Result<Vm, aipo_vm::VmError> {
    let bytecode = compile_program(code);
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);
    vm.run(&bytecode)?;
    Ok(vm)
}

#[test]
fn test_capability_denied() {
    let _guard = CLOCK_LOCK.lock().unwrap();
    revoke_clock();

    let result_now = run_code("var t = time.now()\n");
    let err = match result_now {
        Ok(_) => panic!("time.now() should fault"),
        Err(err) => err,
    };
    assert_eq!(
        err.diagnostic_code(),
        aipo_diagnostics::DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
    );

    let result_monotonic = run_code("var t = time.monotonic()\n");
    let err = match result_monotonic {
        Ok(_) => panic!("time.monotonic() should fault"),
        Err(err) => err,
    };
    assert_eq!(
        err.diagnostic_code(),
        aipo_diagnostics::DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
    );
}

#[test]
fn test_capability_granted() {
    let _guard = CLOCK_LOCK.lock().unwrap();
    install_clock(Box::new(FixedClock {
        wall: 100.0,
        monotonic: 10.0,
    }));

    let vm_now = run_code("var t = time.now()\n").expect("time.now() should succeed");
    let val_now = vm_now.get_global("t").expect("global t exists");
    assert_eq!(val_now, &Value::Duration(100.0));

    let vm_monotonic =
        run_code("var m = time.monotonic()\n").expect("time.monotonic() should succeed");
    let val_monotonic = vm_monotonic.get_global("m").expect("global m exists");
    assert_eq!(val_monotonic, &Value::Duration(10.0));
}

#[test]
fn test_stale_handle_via_host_context() {
    let mut vm = Vm::new();
    let handle = vm.host_context().hand_out(HostValue::Int(42));
    vm.host_context().release(handle);

    let fault = vm.host().resolve(handle).expect_err("should be stale");
    assert_eq!(
        fault.diagnostic_code(),
        aipo_diagnostics::DiagnosticCode::AIPO_RT_STALE_HANDLE
    );
}

#[test]
fn test_stale_handle_across_slot_reuse() {
    let mut vm = Vm::new();
    let handle1 = vm.host_context().hand_out(HostValue::Int(42));
    vm.host_context().release(handle1);

    // Insert new value, reusing the slot but advancing the generation.
    let handle2 = vm.host_context().hand_out(HostValue::Int(100));

    let fault = vm
        .host()
        .resolve(handle1)
        .expect_err("handle1 should still be stale");
    assert_eq!(
        fault.diagnostic_code(),
        aipo_diagnostics::DiagnosticCode::AIPO_RT_STALE_HANDLE
    );

    let value2 = vm.host().resolve(handle2).expect("handle2 should be live");
    assert_eq!(value2, &HostValue::Int(100));
}

#[test]
fn test_handle_resolution_succeeds_for_live_handle() {
    let mut vm = Vm::new();
    let handle = vm.host_context().hand_out(HostValue::string("live"));

    let value = vm.host().resolve(handle).expect("should resolve");
    assert_eq!(value, &HostValue::string("live"));
}
