//! Aipo Virtual Machine: stack-based bytecode interpreter, call frames, value model, and runtime fault handling.

pub mod convert;
pub mod fault;
pub mod frame;
pub mod value;
pub mod vm;

pub use convert::{
    BYTE_MAX, TypeTag, convert_byte, convert_float, convert_int, convert_string, convert_via_type,
};
pub use fault::{VmError, VmFault};
pub use frame::{CallFrame, HandlerFrame};
pub use value::{
    DictMap, FailureValue, MAX_SAFE_INT, MIN_SAFE_INT, MethodKind, StructInstance, Value,
    check_finite_float, check_safe_int,
};
pub use vm::Vm;

use aipo_bytecode::BytecodeModule;

/// Convenience function to instantiate a fresh VM and execute a compiled module.
///
/// # Errors
/// Returns `VmError` if a runtime fault or uncaught failure occurs.
pub fn execute(module: &BytecodeModule) -> Result<Value, VmError> {
    let mut vm = Vm::new();
    vm.run(module)
}
