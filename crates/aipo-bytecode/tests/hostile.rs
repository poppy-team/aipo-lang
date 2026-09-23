//! Hostile bytecode corpus: malformed modules must be rejected with
//! controlled verifier errors — never a panic, hang, or unchecked access.
//!
//! Every case corresponds to a real field of the `aibc` format. Cases that the
//! verifier accepts (valid structure, runtime-level failure) additionally run
//! through the VM inside `catch_unwind` to prove execution itself never panics.

#![forbid(unsafe_code)]

use aipo_bytecode::{AIBC_MAGIC, AIBC_VERSION, BytecodeModule, BytecodeVerifier, Constant, OpCode};
use std::panic::{self, AssertUnwindSafe};

fn base_module() -> BytecodeModule {
    BytecodeModule::new()
}

fn assert_rejected(label: &str, module: &BytecodeModule, fragment: &str) {
    let result = panic::catch_unwind(AssertUnwindSafe(|| BytecodeVerifier::verify(module)));
    let errors = result.unwrap_or_else(|_| panic!("verifier panicked on {label}"));
    let errors = errors.unwrap_err();
    assert!(
        errors.iter().any(|message| message.contains(fragment)),
        "{label}: expected a `{fragment}` rejection, got {errors:?}"
    );
}

#[test]
fn test_verifier_rejects_invalid_opcode() {
    let mut module = base_module();
    module.code = vec![0xFF];
    assert_rejected("invalid-opcode", &module, "invalid opcode byte");
}

#[test]
fn test_verifier_rejects_truncated_instruction() {
    // `Constant` needs a 2-byte operand; only one byte follows.
    let mut module = base_module();
    module.code = vec![OpCode::Constant as u8, 0x00];
    assert_rejected("truncated-instruction", &module, "truncated");
}

#[test]
fn test_verifier_rejects_bad_constant_index() {
    let mut module = base_module();
    module.constants = vec![Constant::Int(1)];
    module.code = vec![OpCode::Constant as u8, 0x00, 0x09, OpCode::Return as u8];
    assert_rejected("bad-constant-index", &module, "out of bounds");
}

#[test]
fn test_verifier_rejects_bad_name_index() {
    let mut module = base_module();
    module.code = vec![OpCode::GetGlobal as u8, 0x00, 0x07, OpCode::Return as u8];
    assert_rejected("bad-name-index", &module, "out of bounds");
}

#[test]
fn test_verifier_rejects_bad_jump_destination() {
    let mut module = base_module();
    // Jump +100 from a 4-byte module: lands outside the code.
    module.code = vec![OpCode::Jump as u8, 0x00, 0x64, OpCode::Return as u8];
    assert_rejected("bad-jump", &module, "out-of-bounds");
}

#[test]
fn test_verifier_rejects_bad_function_index() {
    let mut module = base_module();
    module.code = vec![OpCode::MakeFunction as u8, 0x00, 0x05, OpCode::Return as u8];
    assert_rejected("bad-function-index", &module, "out of bounds");
}

#[test]
fn test_verifier_rejects_bad_handler_target() {
    let mut module = base_module();
    module.code = vec![OpCode::PushHandler as u8, 0x7F, 0xFF, OpCode::Return as u8];
    assert_rejected("bad-handler-target", &module, "out-of-bounds");
}

#[test]
fn test_verifier_rejects_truncated_contract_metadata() {
    let mut module = base_module();
    module.names = vec!["Int".to_string(), "parameter `x`".to_string()];
    // AssertContract header claims one interface operation, but the operation
    // bytes are missing.
    module.code = vec![
        OpCode::AssertContract as u8,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        0x01,
        OpCode::Return as u8,
    ];
    assert_rejected("truncated-contract", &module, "truncated");
}

#[test]
fn test_verifier_rejects_bad_magic_and_version() {
    let mut module = base_module();
    module.magic = *b"NOPE";
    assert_rejected("bad-magic", &module, "invalid magic header");
    let mut module = base_module();
    module.version = AIBC_VERSION + 1;
    assert_rejected("bad-version", &module, "unsupported bytecode version");
    assert_eq!(AIBC_MAGIC, *b"AIBC");
}

#[test]
fn test_well_formed_module_passes_and_executes_without_panic() {
    // `Nil; Return` is the smallest executable module: the verifier accepts it
    // and the VM runs it to `none` without panicking.
    let mut module = base_module();
    module.code = vec![OpCode::Nil as u8, OpCode::Return as u8];
    BytecodeVerifier::verify(&module).expect("minimal module verifies");
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut vm = aipo_vm::Vm::new();
        vm.run(&module)
    }));
    let value = result
        .expect("VM never panics on verified input")
        .expect("runs clean");
    assert!(matches!(value, aipo_vm::Value::None));
}

#[test]
fn test_undefined_global_is_a_fault_not_a_panic() {
    // Structurally valid, semantically failing: `GetGlobal` of a missing name
    // must produce `UndefinedGlobal`, never unwind the host stack.
    let mut module = base_module();
    module.names = vec!["missing".to_string()];
    module.code = vec![OpCode::GetGlobal as u8, 0x00, 0x00, OpCode::Return as u8];
    BytecodeVerifier::verify(&module).expect("undefined global is not structural");
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut vm = aipo_vm::Vm::new();
        vm.run(&module)
    }));
    let outcome = result.expect("VM never panics on undefined globals");
    assert!(outcome.is_err(), "undefined global faults");
}

#[test]
fn test_verifier_rejects_out_of_bounds_local_slot() {
    let mut module = base_module();
    module.functions.push(aipo_bytecode::FunctionInfo {
        name: "__top_level__".to_string(),
        entry_ip: 0,
        params: 0,
        locals: 2,
        upvalues: 0,
        is_async: false,
    });
    // GetLocal 5 when locals limit is 2
    module.code = vec![OpCode::GetLocal as u8, 0x00, 0x05, OpCode::Return as u8];
    assert_rejected(
        "out-of-bounds-local",
        &module,
        "out of bounds for function '__top_level__'",
    );
}

#[test]
fn test_verifier_rejects_out_of_bounds_upvalue_slot() {
    let mut module = base_module();
    module.functions.push(aipo_bytecode::FunctionInfo {
        name: "__top_level__".to_string(),
        entry_ip: 0,
        params: 0,
        locals: 2,
        upvalues: 1,
        is_async: false,
    });
    // GetUpvalue 3 when upvalues limit is 1
    module.code = vec![OpCode::GetUpvalue as u8, 0x00, 0x03, OpCode::Return as u8];
    assert_rejected(
        "out-of-bounds-upvalue",
        &module,
        "out of bounds for function '__top_level__'",
    );
}

#[test]
fn test_verifier_rejects_build_struct_field_count_mismatch() {
    let mut module = base_module();
    module.names = vec!["Point".to_string()];
    module.structs = vec![aipo_bytecode::module::StructInfo {
        name: "Point".to_string(),
        fields: vec![("x".to_string(), false), ("y".to_string(), false)],
    }];
    // BuildStruct Point (type 0), claims 1 field instead of 2
    module.code = vec![
        OpCode::BuildStruct as u8,
        0x00,
        0x00, // type 0: Point
        0x00,
        0x01, // 1 field (mismatch!)
        0x00, // not deferred
        OpCode::Return as u8,
    ];
    assert_rejected(
        "build-struct-field-mismatch",
        &module,
        "BuildStruct for Point expected 2 fields, got 1",
    );
}
