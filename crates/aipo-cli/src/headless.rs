//! Opt-in, in-memory conformance host. No host state is shared between VMs.
//!
//! `scoped` intentionally returns a handle after its native callback's scope closes:
//! fixtures then exercise the VM's real publication checks rather than fabricating faults.

use std::cell::RefCell;
use std::rc::Rc;

use aipo_host::{Capability, Handle, HostValue};
use aipo_sema::PreludeSurface;
use aipo_vm::{DictMap, HostContext, Value, Vm, VmError, VmFault};

const SCHEMA: &str = include_str!("../../../docs/conformance/host/headless_test_host.json");

pub(crate) fn surface() -> Result<PreludeSurface, String> {
    crate::ahs::from_json(SCHEMA)
        .map_err(|fault| format!("invalid built-in headless schema: {fault}"))
}

pub(crate) fn install(vm: &mut Vm) {
    *vm.host_context() = HostContext::denied();
    let natives: [(&str, aipo_vm::HostNativeCallback); 4] = [
        ("create", create),
        ("read", read),
        ("release", release),
        ("scoped", scoped),
    ];
    let entries = natives
        .into_iter()
        .map(|(member, callback)| {
            let name = format!("headless.{member}");
            vm.register_host_native(&name, 1, callback);
            (
                Value::String(Rc::new(member.to_string())),
                Value::native(name, 1, unavailable),
            )
        })
        .collect();
    vm.define_global(
        "headless",
        Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries)))),
    );
    // Override only this VM's callbacks: do not revoke the process-global CLI clock.
    vm.register_host_native("time.now", 0, denied_wall_clock);
    vm.register_host_native("time.monotonic", 0, denied_monotonic_clock);
}

fn unavailable(_args: &[Value]) -> Result<Value, VmFault> {
    Err(VmFault::CorruptedBytecode {
        offset: 0,
        reason: "headless native called without its VM context".to_string(),
    })
}

fn integer(args: &[Value]) -> Result<i64, VmError> {
    match args {
        [Value::Int(value)] => Ok(*value),
        _ => Err(VmFault::TypeMismatch {
            expected: "one Int value".to_string(),
            actual: args.first().map_or("missing", Value::type_name).to_string(),
        }
        .into()),
    }
}

fn handle(args: &[Value]) -> Result<Handle, VmError> {
    match args {
        [Value::HostHandle(handle)] => Ok(*handle),
        _ => Err(VmFault::TypeMismatch {
            expected: "one host Handle".to_string(),
            actual: args.first().map_or("missing", Value::type_name).to_string(),
        }
        .into()),
    }
}

fn create(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let value = integer(args)?;
    Ok(Value::HostHandle(
        vm.host_context().hand_out(HostValue::Int(value)),
    ))
}

fn read(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let handle = handle(args)?;
    Ok(aipo_vm::host_value_to_value(vm.host().resolve(handle)?)?)
}

fn release(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let handle = handle(args)?;
    vm.host().resolve(handle)?;
    vm.host_context().release(handle);
    Ok(Value::None)
}

fn scoped(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let value = integer(args)?;
    let scope = vm.host_context().open_scope();
    let handle = vm.host_context().hand_out(HostValue::Int(value));
    vm.host_context().close_scope(scope);
    Ok(Value::HostHandle(handle))
}

fn clock(vm: &Vm, operation: &str) -> Result<Value, VmError> {
    let capability =
        Capability::parse(Capability::CLOCK).map_err(|error| VmFault::CorruptedBytecode {
            offset: 0,
            reason: format!("invalid clock capability: {error}"),
        })?;
    vm.host().require(&capability, operation)?;
    // No clock source is installed even if an embedding caller later grants a capability.
    Err(VmFault::CapabilityDenied {
        capability: Capability::CLOCK.to_string(),
        operation: operation.to_string(),
    }
    .into())
}

fn denied_wall_clock(vm: &mut Vm, _args: &[Value]) -> Result<Value, VmError> {
    clock(vm, "time.now")
}

fn denied_monotonic_clock(vm: &mut Vm, _args: &[Value]) -> Result<Value, VmError> {
    clock(vm, "time.monotonic")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_and_runtime_registration_agree() {
        let surface = surface().expect("valid built-in schema");
        let mut vm = Vm::new();
        install(&mut vm);
        for (module, functions) in surface.host_modules() {
            assert!(vm.get_global(module).is_some());
            for (member, signature) in functions {
                let native = vm
                    .host_native(&format!("{module}.{member}"))
                    .expect("registered native");
                assert_eq!(native.arity, signature.params.len());
                assert!(!native.is_async);
            }
        }
        assert!(vm.host().granted().is_empty());
        assert!(!vm.host().has_environment_source());
        assert!(!vm.host().has_filesystem_source());
    }

    #[test]
    fn contexts_are_isolated_and_scope_closure_releases_handles() {
        let mut first = Vm::new();
        let mut second = Vm::new();
        install(&mut first);
        install(&mut second);
        let value = create(&mut first, &[Value::Int(42)]).expect("create");
        assert_eq!(first.host().live_objects(), 1);
        assert_eq!(second.host().live_objects(), 0);
        assert_eq!(read(&mut first, &[value]).expect("read"), Value::Int(42));
        let escaped = scoped(&mut first, &[Value::Int(7)]).expect("scoped");
        assert_eq!(first.host().live_objects(), 1);
        assert!(
            first
                .host()
                .ensure_publishable(&escaped, "test publication")
                .is_err()
        );
        assert!(!second.host().has_escapes());
    }
}
