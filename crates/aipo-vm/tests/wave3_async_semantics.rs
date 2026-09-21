//! Async call protocol and severity regressions for Wave 3 (`P02-G03`).
//!
//! These pin the semantics that the `async fn` surface depends on and that a
//! differential fixture cannot localize:
//!
//! - calling an `async fn` (free, closure or `impl` method) yields a `Task`
//!   instead of running the body;
//! - a recoverable `Failure` produced inside a task travels out through `await`
//!   and is captured by `attempt` like any other failure;
//! - a *fault* raised inside a driven task keeps its own code and ends the
//!   program (canon never lets a fault become a capturable `Failure`);
//! - cancellation ends a task as cancelled, and awaiting it faults.

use aipo_bytecode::compile;
use aipo_hir::lower;
use aipo_ir::lower_to_ir;
use aipo_runtime::NativeRegistry;
use aipo_source::{Source, SourceId};
use aipo_stdlib::register_stdlib;
use aipo_syntax::parse;
use aipo_vm::{Value, Vm, VmError, VmFault};

fn build(code: &str) -> (Vm, aipo_bytecode::BytecodeModule) {
    let src = Source::new(SourceId::next(), "test.aipo", code);
    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {parse_diags:?}");
    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let module = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);
    // Struct layouts travel with the module: without them the runtime cannot name the
    // fields of a user-declared instance.
    for decl in &module.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    // `impl` methods reach the runtime through the module's function table, the way the
    // CLI and the testkit register them.
    for function in &module.functions {
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
    (vm, module)
}

fn run(code: &str) -> Vm {
    let (mut vm, module) = build(code);
    vm.run(&module).expect("program runs to completion");
    vm
}

/// Runs a program that must stop with an error (`Vm` itself has no `Debug` impl).
fn run_error(code: &str) -> VmError {
    let (mut vm, module) = build(code);
    match vm.run(&module) {
        Ok(_) => panic!("program was expected to stop with an error"),
        Err(error) => error,
    }
}

#[test]
fn test_async_fn_call_yields_a_task_and_await_returns_the_value() {
    let vm = run(r#"
        async fn double(x)
            return x * 2
        end

        let t = double(21)
        let value = await t
        "#);

    assert!(
        matches!(vm.get_global("t"), Some(Value::Task(_))),
        "calling an `async fn` must yield a Task handle"
    );
    assert_eq!(vm.get_global("value"), Some(&Value::Int(42)));
}

#[test]
fn test_async_method_call_yields_a_task_carrying_the_receiver() {
    let vm = run(r#"
        struct Counter
            value
        end

        impl Counter
            async fn doubled(self)
                return self.value * 2
            end
        end

        let counter = Counter{value = 21}
        let t = counter.doubled()
        let value = await t
        "#);

    assert!(
        matches!(vm.get_global("t"), Some(Value::Task(_))),
        "calling an `async fn` method must yield a Task handle"
    );
    assert_eq!(vm.get_global("value"), Some(&Value::Int(42)));
}

#[test]
fn test_async_closure_call_yields_a_task() {
    let vm = run(r#"
        let increment = async fn (n)
            return n + 1
        end

        let t = increment(41)
        let value = await t
        "#);

    assert_eq!(vm.get_global("value"), Some(&Value::Int(42)));
}

#[test]
fn test_failure_produced_inside_a_task_is_captured_by_attempt() {
    let vm = run(r#"
        async fn risky()
            return fail("boom")
        end

        var outcome = "pending"
        attempt
            let t = risky()
            let value = await t
            outcome = String(value)
        failed err
            outcome = f"caught: {err.message}"
        end
        "#);

    assert_eq!(
        vm.get_global("outcome"),
        Some(&Value::String(std::rc::Rc::new("caught: boom".to_string())))
    );
}

#[test]
fn test_failure_escaping_a_task_ends_the_program_as_uncaught() {
    let error = run_error(
        r#"
        async fn risky()
            return fail("boom")
        end

        let t = risky()
        let value = await t
        "#,
    );

    assert_eq!(error, VmError::UncaughtFailure("boom".to_string()));
}

#[test]
fn test_fault_inside_a_driven_task_keeps_its_own_code() {
    // A division by zero inside a task is a fault (never a `Failure`), so it must reach the
    // caller as `AIPO_RT_DIV_ZERO` instead of being downgraded to a capturable value.
    let error = run_error(
        r#"
        async fn div_zero()
            return 1 div 0
        end

        var outcome = "pending"
        attempt
            let t = div_zero()
            let value = await t
            outcome = String(value)
        failed err
            outcome = f"caught: {err.message}"
        end
        "#,
    );

    assert_eq!(
        error,
        VmError::Fault(VmFault::DivisionByZero),
        "the task's fault must keep its code"
    );
}

#[test]
fn test_await_cycle_between_tasks_is_a_fault() {
    let error = run_error(
        r#"
        var first_holder = 0
        var second_holder = 0

        async fn first()
            let other = await second_holder
            return other
        end

        async fn second()
            let other = await first_holder
            return other
        end

        first_holder = first()
        second_holder = second()
        let done = await first_holder
        "#,
    );

    assert!(
        matches!(error, VmError::Fault(VmFault::AwaitCycle { .. })),
        "expected an await cycle fault, got {error:?}"
    );
}

#[test]
fn test_awaiting_a_cancelled_task_faults_with_cancelled() {
    let error = run_error(
        r#"
        async fn worker()
            task.sleep(5)
            return 1
        end

        let t = worker()
        task.cancel(t)
        let value = await t
        "#,
    );

    assert!(
        matches!(error, VmError::Fault(VmFault::Cancelled { .. })),
        "expected a cancellation fault, got {error:?}"
    );
}
