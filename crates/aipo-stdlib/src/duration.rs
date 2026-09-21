//! Canonical `Duration` APIs.

use aipo_vm::{Value, Vm, VmFault};

fn require_arity(args: &[Value], expected: usize, op: &str) -> Result<(), VmFault> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(VmFault::TypeMismatch {
            expected: format!("{expected} argument(s) for {op}"),
            actual: format!("{} arguments", args.len()),
        })
    }
}

/// `duration.total_seconds()` — returns duration in seconds as a Float.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if receiver is not Duration.
pub fn duration_total_seconds(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "duration.total_seconds")?;
    match receiver {
        Value::Duration(s) => Ok(Value::Float(*s)),
        other => Err(VmFault::TypeMismatch {
            expected: "Duration receiver".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Registers Duration methods on the VM.
pub fn register_methods(vm: &mut Vm) {
    vm.register_method_native("Duration", "total_seconds", 0, duration_total_seconds);
}
