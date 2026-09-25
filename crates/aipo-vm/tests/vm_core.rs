//! Comprehensive unit tests for VM Core (Slice S7).

use aipo_bytecode::opcode::{Constant, OpCode};
use aipo_bytecode::{BytecodeModule, FunctionInfo};
use aipo_diagnostics::DiagnosticCode;
use aipo_vm::{MAX_SAFE_INT, MIN_SAFE_INT, Value, Vm, VmError, VmFault, execute};
use byteorder::{BigEndian, ByteOrder};

fn make_test_module(code: Vec<u8>, constants: Vec<Constant>, names: Vec<String>) -> BytecodeModule {
    let mut module = BytecodeModule::new();
    module.code = code;
    module.constants = constants;
    module.names = names;
    module
}

#[test]
fn test_constant_loading_and_stack_primitives() {
    let mut code = Vec::new();
    // Constant 0: Int(42)
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    // Dup -> [42, 42]
    code.push(OpCode::Dup as u8);
    // Pop -> [42]
    code.push(OpCode::Pop as u8);
    // True -> [42, true]
    code.push(OpCode::True as u8);
    // Pop -> [42]
    code.push(OpCode::Pop as u8);
    // Nil -> [42, none]
    code.push(OpCode::Nil as u8);
    // Pop -> [42]
    code.push(OpCode::Pop as u8);

    let module = make_test_module(code, vec![Constant::Int(42)], vec![]);
    let result = execute(&module).expect("execution should succeed");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_safe_integer_arithmetic() {
    let mut code = Vec::new();
    // Load 10 (idx 0), Load 20 (idx 1), Add -> 30
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Add as u8);

    // Load 5 (idx 2), Sub -> 25
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&2u16.to_be_bytes());
    code.push(OpCode::Sub as u8);

    // Load 2 (idx 3), Mul -> 50
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&3u16.to_be_bytes());
    code.push(OpCode::Mul as u8);

    // Load 3 (idx 4), IntDiv -> 16
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&4u16.to_be_bytes());
    code.push(OpCode::IntDiv as u8);

    // Load 5 (idx 2), Mod -> 1
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&2u16.to_be_bytes());
    code.push(OpCode::Mod as u8);

    // Neg -> -1
    code.push(OpCode::Neg as u8);

    let module = make_test_module(
        code,
        vec![
            Constant::Int(10),
            Constant::Int(20),
            Constant::Int(5),
            Constant::Int(2),
            Constant::Int(3),
        ],
        vec![],
    );

    let result = execute(&module).expect("execution should succeed");
    assert_eq!(result, Value::Int(-1));
}

#[test]
fn test_integer_overflow_fault_at_boundary() {
    let mut code = Vec::new();
    // Load MAX_SAFE_INT (idx 0), Load 1 (idx 1), Add
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Add as u8);

    let module = make_test_module(
        code,
        vec![Constant::Int(MAX_SAFE_INT), Constant::Int(1)],
        vec![],
    );

    let err = execute(&module).expect_err("overflow must trigger fault");
    assert_eq!(err.diagnostic_code(), DiagnosticCode::AIPO_RT_OVERFLOW);
    assert!(matches!(err, VmError::Fault(VmFault::Overflow { .. })));
}

#[test]
fn test_integer_underflow_fault_at_boundary() {
    let mut code = Vec::new();
    // Load MIN_SAFE_INT (idx 0), Load 1 (idx 1), Sub
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Sub as u8);

    let module = make_test_module(
        code,
        vec![Constant::Int(MIN_SAFE_INT), Constant::Int(1)],
        vec![],
    );

    let err = execute(&module).expect_err("underflow must trigger fault");
    assert_eq!(err.diagnostic_code(), DiagnosticCode::AIPO_RT_OVERFLOW);
}

#[test]
fn test_division_by_zero_fault() {
    let mut code = Vec::new();
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::IntDiv as u8);

    let module = make_test_module(code, vec![Constant::Int(10), Constant::Int(0)], vec![]);
    let err = execute(&module).expect_err("div by zero must trigger fault");
    assert_eq!(err.diagnostic_code(), DiagnosticCode::AIPO_RT_DIV_ZERO);
    assert_eq!(err, VmError::Fault(VmFault::DivisionByZero));
}

#[test]
fn test_float_finite_and_division() {
    let mut code = Vec::new();
    // 7.0 / 2.0 = 3.5
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Div as u8);

    let module = make_test_module(
        code,
        vec![Constant::Float(7.0), Constant::Float(2.0)],
        vec![],
    );
    let result = execute(&module).expect("float div should succeed");
    assert_eq!(result, Value::Float(3.5));
}

#[test]
fn test_float_division_by_zero_fault() {
    let mut code = Vec::new();
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Div as u8);

    let module = make_test_module(
        code,
        vec![Constant::Float(5.0), Constant::Float(0.0)],
        vec![],
    );
    let err = execute(&module).expect_err("float div zero must trigger fault");
    assert_eq!(err.diagnostic_code(), DiagnosticCode::AIPO_RT_DIV_ZERO);
}

#[test]
fn test_comparisons_and_boolean_logic() {
    let mut code = Vec::new();
    // 10 < 20 -> true
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Less as u8);

    // not true -> false
    code.push(OpCode::Not as u8);

    let module = make_test_module(code, vec![Constant::Int(10), Constant::Int(20)], vec![]);
    let result = execute(&module).expect("comparison should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_jump_and_branching() {
    let mut code = Vec::new();
    // push false
    code.push(OpCode::False as u8);
    // JumpIfFalse +6 (skips push 10)
    code.push(OpCode::JumpIfFalse as u8);
    let mut buf = [0u8; 2];
    BigEndian::write_i16(&mut buf, 6);
    code.extend_from_slice(&buf);

    // push 10 (3 bytes)
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    // jump +3 (skips push 20)
    code.push(OpCode::Jump as u8);
    BigEndian::write_i16(&mut buf, 3);
    code.extend_from_slice(&buf);

    // push 20 (3 bytes)
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());

    let module = make_test_module(code, vec![Constant::Int(10), Constant::Int(20)], vec![]);
    let result = execute(&module).expect("branching should succeed");
    assert_eq!(result, Value::Int(20));
}

#[test]
fn test_global_variables() {
    let mut code = Vec::new();
    // Constant 0: Int(100)
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    // SetGlobal "counter" (name idx 0)
    code.push(OpCode::SetGlobal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());

    // GetGlobal "counter" (name idx 0)
    code.push(OpCode::GetGlobal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    // Constant 1: Int(50)
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    // Add -> 150
    code.push(OpCode::Add as u8);

    let module = make_test_module(
        code,
        vec![Constant::Int(100), Constant::Int(50)],
        vec!["counter".to_string()],
    );

    let mut vm = Vm::new();
    let result = vm.run(&module).expect("execution should succeed");
    assert_eq!(result, Value::Int(150));
    assert_eq!(vm.get_global("counter"), Some(&Value::Int(100)));
}

#[test]
fn test_function_call_and_return() {
    let mut vm = Vm::new();
    // Define the function value in globals
    vm.define_global(
        "double",
        Value::Function {
            entry_ip: 3,
            arity: 1,
            is_async: false,
        },
    );

    let mut code = Vec::new();
    let mut buf = [0u8; 2];
    // Offset 0: Jump past function body to main (@ 11)
    code.push(OpCode::Jump as u8);
    BigEndian::write_i16(&mut buf, 8);
    code.extend_from_slice(&buf);

    // Function body @ 3: GetLocal 0, Mul 2, Return
    code.push(OpCode::GetLocal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Mul as u8);
    code.push(OpCode::Return as u8);

    // Main @ 11: GetGlobal "double", Push 21, Call 1
    code.push(OpCode::GetGlobal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Call as u8);
    code.push(1u8);

    let module = make_test_module(
        code,
        vec![Constant::Int(2), Constant::Int(21)],
        vec!["double".to_string()],
    );

    let result = vm.run(&module).expect("function call should succeed");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_nested_calls_restore_the_outer_frame_base() {
    let mut vm = Vm::new();
    vm.define_global(
        "inner",
        Value::Function {
            entry_ip: 20,
            arity: 1,
            is_async: false,
        },
    );
    vm.define_global(
        "outer",
        Value::Function {
            entry_ip: 3,
            arity: 1,
            is_async: false,
        },
    );

    let mut code = Vec::new();
    // Main @ 28: call outer(10), then return its result.
    code.push(OpCode::Jump as u8);
    code.extend_from_slice(&25i16.to_be_bytes());

    // outer @ 3: inner(local 0), discard its result, then add 10 to local 0.
    code.push(OpCode::GetGlobal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::GetLocal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Call as u8);
    code.push(1u8);
    code.push(OpCode::Pop as u8);
    code.push(OpCode::GetLocal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Add as u8);
    code.push(OpCode::Return as u8);

    // inner @ 20: local 0 * 2.
    code.push(OpCode::GetLocal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Mul as u8);
    code.push(OpCode::Return as u8);

    // Main @ 28: outer(10).
    code.push(OpCode::GetGlobal as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Call as u8);
    code.push(1u8);
    code.push(OpCode::Return as u8);

    let module = make_test_module(
        code,
        vec![Constant::Int(2), Constant::Int(10)],
        vec!["inner".to_string(), "outer".to_string()],
    );

    let result = vm.run(&module).expect("nested calls should succeed");
    assert_eq!(result, Value::Int(20));
}

#[test]
fn test_call_with_extra_arguments_faults_on_arity() {
    // Regression guard for the runtime arity check: a callee declared with one
    // parameter must reject two arguments even when every value is well-typed.
    // (Static arity is checked by sema; this is the last line of defense for
    // hand-built bytecode.)
    let mut vm = Vm::new();
    vm.define_global(
        "only_one",
        Value::Function {
            entry_ip: 0,
            arity: 1,
            is_async: false,
        },
    );

    let mut code = Vec::new();
    // GetGlobal "only_one", Constant 1, Constant 2, Call 2, Return
    code.push(OpCode::GetGlobal as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&0u16.to_be_bytes());
    code.push(OpCode::Constant as u8);
    code.extend_from_slice(&1u16.to_be_bytes());
    code.push(OpCode::Call as u8);
    code.push(2u8);
    code.push(OpCode::Return as u8);

    let mut module = make_test_module(
        code,
        vec![Constant::Int(1), Constant::Int(2)],
        vec!["only_one".to_string()],
    );
    module.functions.push(FunctionInfo {
        name: "only_one".to_string(),
        entry_ip: 0,
        params: 1,
        locals: 0,
        upvalues: 0,
        is_async: false,
    });

    let outcome = vm.run(&module);
    assert!(
        matches!(outcome, Err(VmError::Fault(VmFault::TypeMismatch { .. }))),
        "extra arguments fault, got {outcome:?}"
    );
}

fn host_context_value(_vm: &mut Vm, _args: &[Value]) -> Result<Value, VmError> {
    Ok(Value::Int(99))
}

fn host_context_reads_vm(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        }
        .into());
    }
    Ok(vm.get_global("context").cloned().unwrap_or(Value::None))
}

fn unused_native(_args: &[Value]) -> Result<Value, VmFault> {
    Err(VmFault::CorruptedBytecode {
        offset: 0,
        reason: "ordinary native unexpectedly ran".to_string(),
    })
}

#[test]
fn test_host_native_dispatches_with_vm_context() {
    let mut vm = Vm::new();
    vm.define_global(
        "context",
        Value::String(std::rc::Rc::new("per-vm".to_string())),
    );
    vm.define_global(
        "read_context",
        Value::Native {
            name: "test.read_context".to_string(),
            arity: 1,
            func: unused_native,
        },
    );
    vm.register_host_native("test.read_context", 1, host_context_reads_vm);

    let module = make_test_module(
        vec![
            OpCode::GetGlobal as u8,
            0,
            0,
            OpCode::Constant as u8,
            0,
            0,
            OpCode::Call as u8,
            1,
            OpCode::Return as u8,
        ],
        vec![Constant::String("argument".to_string())],
        vec!["read_context".to_string()],
    );

    let result = vm.run(&module).expect("host native should run");
    assert_eq!(
        result,
        Value::String(std::rc::Rc::new("per-vm".to_string()))
    );
}

#[test]
fn test_ordinary_native_still_dispatches_without_a_host_callback() {
    let mut vm = Vm::new();
    vm.define_global(
        "ordinary",
        Value::Native {
            name: "test.ordinary".to_string(),
            arity: 1,
            func: |args| {
                let Some(&Value::Int(value)) = args.first() else {
                    return Err(VmFault::TypeMismatch {
                        expected: "Int".to_string(),
                        actual: args.first().map_or("none", Value::type_name).to_string(),
                    });
                };
                Ok(Value::Int(value + 1))
            },
        },
    );

    let module = make_test_module(
        vec![
            OpCode::GetGlobal as u8,
            0,
            0,
            OpCode::Constant as u8,
            0,
            0,
            OpCode::Call as u8,
            1,
            OpCode::Return as u8,
        ],
        vec![Constant::Int(41)],
        vec!["ordinary".to_string()],
    );

    assert_eq!(
        vm.run(&module).expect("ordinary native should run"),
        Value::Int(42)
    );
}

#[test]
fn test_host_native_error_preserves_the_operand_stack() {
    let mut vm = Vm::new();
    let callee = Value::Native {
        name: "test.host_error".to_string(),
        arity: 1,
        func: unused_native,
    };
    let argument = Value::Int(7);
    vm.stack = vec![callee.clone(), argument.clone()];
    vm.ip = 0;
    vm.register_host_native("test.host_error", 1, |_vm, _args| {
        Err(VmFault::DivisionByZero.into())
    });

    let module = make_test_module(vec![OpCode::Call as u8, 1], vec![], vec![]);
    let error = vm.step(&module).expect_err("host native should fault");
    assert_eq!(error.diagnostic_code(), DiagnosticCode::AIPO_RT_DIV_ZERO);
    assert_eq!(vm.stack, vec![callee, argument]);
}

#[test]
fn test_host_native_arity_mismatch_faults_without_calling_the_placeholder() {
    let mut vm = Vm::new();
    let callee = Value::Native {
        name: "test.arity".to_string(),
        arity: 2,
        func: unused_native,
    };
    let first = Value::Int(1);
    let second = Value::Int(2);
    vm.stack = vec![callee.clone(), first.clone(), second.clone()];
    vm.ip = 0;
    vm.register_host_native("test.arity", 1, host_context_value);

    let module = make_test_module(vec![OpCode::Call as u8, 2], vec![], vec![]);
    let error = vm
        .step(&module)
        .expect_err("registered arity must be checked");
    assert!(matches!(
        error,
        VmError::Fault(VmFault::TypeMismatch { .. })
    ));
    assert_eq!(vm.stack, vec![callee, first, second]);
}

fn host_async_value(vm: &mut Vm, _args: &[Value]) -> Result<Value, VmError> {
    let count = match vm.get_global("host_count") {
        Some(Value::Int(value)) => *value,
        _ => 0,
    };
    vm.define_global("host_count", Value::Int(count + 1));
    Ok(Value::Int(7))
}

fn host_async_fault(_vm: &mut Vm, _args: &[Value]) -> Result<Value, VmError> {
    Err(VmFault::DivisionByZero.into())
}

fn async_native_module(await_result: bool) -> BytecodeModule {
    let mut code = vec![
        OpCode::GetGlobal as u8,
        0,
        0,
        OpCode::Constant as u8,
        0,
        0,
        OpCode::Call as u8,
        1,
    ];
    if await_result {
        code.push(OpCode::Await as u8);
    }
    code.push(OpCode::Return as u8);
    make_test_module(
        code,
        vec![Constant::Int(1)],
        vec!["async_native".to_string()],
    )
}

fn define_async_native(vm: &mut Vm) {
    vm.define_global(
        "async_native",
        Value::Native {
            name: "test.async_native".to_string(),
            arity: 1,
            func: unused_native,
        },
    );
    vm.register_host_async_native("test.async_native", 1, host_async_value);
}

#[test]
fn test_async_host_native_returns_task_before_scheduler_runs_callback() {
    let mut vm = Vm::new();
    define_async_native(&mut vm);
    let result = vm
        .run(&async_native_module(false))
        .expect("main task completes without driving forgotten task");
    assert!(matches!(result, Value::Task(_)));
    assert_eq!(vm.get_global("host_count"), None);
}

#[test]
fn test_async_host_native_awaits_and_runs_on_scheduler() {
    let mut vm = Vm::new();
    define_async_native(&mut vm);
    let result = vm
        .run(&async_native_module(true))
        .expect("host task resolves");
    assert_eq!(result, Value::Int(7));
    assert_eq!(vm.get_global("host_count"), Some(&Value::Int(1)));
}

#[test]
fn test_async_host_native_error_keeps_runtime_fault() {
    let mut vm = Vm::new();
    vm.define_global(
        "async_native",
        Value::Native {
            name: "test.async_native".to_string(),
            arity: 1,
            func: unused_native,
        },
    );
    vm.register_host_async_native("test.async_native", 1, host_async_fault);
    let error = vm
        .run(&async_native_module(true))
        .expect_err("host task error remains a fault");
    assert_eq!(
        error.diagnostic_code(),
        DiagnosticCode::AIPO_RT_DIV_ZERO,
        "{error:?}"
    );
}

#[test]
fn test_async_host_native_can_be_cancelled_before_driving() {
    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    define_async_native(&mut vm);
    let code = vec![
        OpCode::GetGlobal as u8,
        0,
        0,
        OpCode::Constant as u8,
        0,
        0,
        OpCode::Call as u8,
        1,
        OpCode::SetGlobal as u8,
        0,
        3,
        OpCode::GetGlobal as u8,
        0,
        1,
        OpCode::GetField as u8,
        0,
        2,
        OpCode::GetGlobal as u8,
        0,
        3,
        OpCode::Call as u8,
        1,
        OpCode::Pop as u8,
        OpCode::GetGlobal as u8,
        0,
        3,
        OpCode::Await as u8,
        OpCode::Return as u8,
    ];
    let module = make_test_module(
        code,
        vec![Constant::Int(1)],
        vec![
            "async_native".to_string(),
            "task".to_string(),
            "cancel".to_string(),
            "saved_task".to_string(),
        ],
    );
    let error = vm
        .run(&module)
        .expect_err("cancelled host task faults when awaited");
    assert_eq!(
        error.diagnostic_code(),
        DiagnosticCode::AIPO_RT_CANCELLED,
        "{error:?}"
    );
    assert_eq!(vm.get_global("host_count"), None);
}

#[test]
fn test_metrics_count_executed_instructions_and_constants() {
    let mut vm = Vm::new();
    vm.enable_metrics();
    let module = make_test_module(
        vec![OpCode::Constant as u8, 0, 0],
        vec![Constant::Int(7)],
        vec![],
    );
    vm.run(&module).expect("metric workload runs");
    let metrics = vm.metrics();
    assert_eq!(metrics.instructions, 2);
    assert_eq!(metrics.constant_loads, 1);
}
