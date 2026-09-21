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
        is_async: false,
    });

    let outcome = vm.run(&module);
    assert!(
        matches!(outcome, Err(VmError::Fault(VmFault::TypeMismatch { .. }))),
        "extra arguments fault, got {outcome:?}"
    );
}
