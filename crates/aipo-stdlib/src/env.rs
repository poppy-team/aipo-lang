//! Capability-aware environment access for Aipo hosts.
//!
//! The module exposes only the synchronous read operations `env.get` and `env.has`. Both
//! resolve against the provider installed in the current VM's [`HostContext`](aipo_vm::HostContext),
//! require `env.read`, and fail with the VM capability fault when either condition is absent.

use aipo_host::Capability;
use aipo_vm::{DictMap, EnvironmentSource, Value, Vm, VmError, VmFault};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// A deterministic environment provider backed by an ordered map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapEnvironmentSource {
    values: BTreeMap<String, String>,
}

impl MapEnvironmentSource {
    /// Creates an empty provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a provider from name/value pairs.
    #[must_use]
    pub fn from_entries<I, K, V>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            values: entries
                .into_iter()
                .map(|(name, value)| (name.into(), value.into()))
                .collect(),
        }
    }

    /// Creates a provider from name/value pairs.
    #[must_use]
    pub fn with_entries<I, K, V>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::from_entries(entries)
    }

    /// Inserts or replaces one deterministic environment value.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.values.insert(name.into(), value.into())
    }
}

impl<K, V> FromIterator<(K, V)> for MapEnvironmentSource
where
    K: Into<String>,
    V: Into<String>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        Self::from_entries(iter)
    }
}

impl EnvironmentSource for MapEnvironmentSource {
    fn get(&self, name: &str) -> Option<String> {
        self.values.get(name).cloned()
    }
}

/// Installs an environment provider in one VM without granting `env.read`.
pub fn install_environment_source<S>(vm: &mut Vm, source: S)
where
    S: EnvironmentSource + 'static,
{
    vm.host_context().install_environment_source(source);
}

/// Installs an environment provider in one VM without granting `env.read`.
pub fn install_environment<S>(vm: &mut Vm, source: S)
where
    S: EnvironmentSource + 'static,
{
    install_environment_source(vm, source);
}

fn environment_capability() -> Result<Capability, VmError> {
    Capability::parse(Capability::ENV_READ).map_err(|error| {
        VmFault::CorruptedBytecode {
            offset: 0,
            reason: format!("invalid env.read capability: {error}"),
        }
        .into()
    })
}

fn require_environment(vm: &Vm, operation: &str) -> Result<(), VmError> {
    let capability = environment_capability()?;
    vm.host().require(&capability, operation)?;
    if vm.host().environment_source().is_none() {
        return Err(VmFault::CapabilityDenied {
            capability: Capability::ENV_READ.to_string(),
            operation: operation.to_string(),
        }
        .into());
    }
    Ok(())
}

fn read_name(args: &[Value], operation: &str) -> Result<String, VmError> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: format!("1 argument for {operation}"),
            actual: format!("{} arguments", args.len()),
        }
        .into());
    }
    match &args[0] {
        Value::String(name) => Ok(name.as_str().to_string()),
        other => Err(VmFault::TypeMismatch {
            expected: format!("String name for {operation}"),
            actual: other.type_name().to_string(),
        }
        .into()),
    }
}

/// Reads an environment value, returning `none` when the name is absent.
///
/// # Errors
///
/// Returns [`VmFault::CapabilityDenied`] when `env.read` is not granted or no provider is
/// installed, and [`VmFault::TypeMismatch`] for an invalid name argument.
pub fn get(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let name = read_name(args, "env.get")?;
    require_environment(vm, "env.get")?;
    let value = vm
        .host()
        .environment_source()
        .and_then(|source| source.get(&name));
    Ok(value.map_or(Value::None, |value| Value::String(Rc::new(value))))
}

/// Checks whether an environment name is present.
///
/// # Errors
///
/// Returns [`VmFault::CapabilityDenied`] when `env.read` is not granted or no provider is
/// installed, and [`VmFault::TypeMismatch`] for an invalid name argument.
pub fn has(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let name = read_name(args, "env.has")?;
    require_environment(vm, "env.has")?;
    Ok(Value::Bool(
        vm.host()
            .environment_source()
            .is_some_and(|source| source.get(&name).is_some()),
    ))
}

/// Alias for [`get`].
pub fn env_get(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    get(vm, args)
}

/// Alias for [`has`].
pub fn env_has(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    has(vm, args)
}

fn unavailable(_args: &[Value]) -> Result<Value, VmFault> {
    Err(VmFault::CorruptedBytecode {
        offset: 0,
        reason: "environment native was called without its VM callback".to_string(),
    })
}

/// Registers the VM-context-aware environment callbacks.
pub fn register_natives(vm: &mut Vm) {
    vm.register_host_native("env.get", 1, get);
    vm.register_host_native("env.has", 1, has);
}

/// Creates the canonical `env` module dictionary.
#[must_use]
pub fn create_module() -> Value {
    let entries = vec![
        (
            Value::String(Rc::new("get".to_string())),
            Value::native("env.get", 1, unavailable),
        ),
        (
            Value::String(Rc::new("has".to_string())),
            Value::native("env.has", 1, unavailable),
        ),
    ];
    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register_stdlib;
    use aipo_bytecode::BytecodeModule;
    use aipo_bytecode::opcode::{Constant, OpCode};
    use aipo_diagnostics::DiagnosticCode;
    use aipo_runtime::NativeRegistry;

    fn make_vm() -> (Vm, NativeRegistry) {
        let mut vm = Vm::new();
        let mut registry = NativeRegistry::new();
        register_stdlib(&mut vm, &mut registry);
        (vm, registry)
    }

    fn grant_env(vm: &mut Vm) {
        vm.host_context().grant(
            Capability::parse(Capability::ENV_READ).expect("env.read is a valid capability"),
        );
    }

    fn name(value: &str) -> Value {
        Value::String(Rc::new(value.to_string()))
    }

    fn run_env_call(vm: &mut Vm, function: &str, argument: &str) -> Result<Value, VmError> {
        let mut module = BytecodeModule::new();
        module.names = vec!["env".to_string(), function.to_string()];
        module.constants = vec![Constant::String(argument.to_string())];
        module.code = vec![
            OpCode::GetGlobal as u8,
            0,
            0,
            OpCode::GetField as u8,
            0,
            1,
            OpCode::Constant as u8,
            0,
            0,
            OpCode::Call as u8,
            1,
            OpCode::Return as u8,
        ];
        vm.run(&module)
    }

    #[test]
    fn test_env_get_is_denied_without_grant() {
        let (mut vm, _registry) = make_vm();
        install_environment_source(
            &mut vm,
            MapEnvironmentSource::from_entries([("AIPO_TEST", "value")]),
        );

        let error = run_env_call(&mut vm, "get", "AIPO_TEST").expect_err("grant is required");
        assert_eq!(
            error.diagnostic_code(),
            DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
        );
    }

    #[test]
    fn test_env_get_is_denied_without_provider() {
        let (mut vm, _registry) = make_vm();
        grant_env(&mut vm);

        let error = run_env_call(&mut vm, "get", "AIPO_TEST").expect_err("provider is required");
        assert_eq!(
            error.diagnostic_code(),
            DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
        );
    }

    #[test]
    fn test_env_get_and_has_return_copied_values() {
        let (mut vm, _registry) = make_vm();
        install_environment_source(
            &mut vm,
            MapEnvironmentSource::from_entries([("AIPO_TEST", "value")]),
        );
        grant_env(&mut vm);

        assert_eq!(
            run_env_call(&mut vm, "get", "AIPO_TEST").expect("get succeeds"),
            name("value")
        );
        assert_eq!(
            run_env_call(&mut vm, "has", "AIPO_TEST").expect("has succeeds"),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_env_missing_name_is_none_or_false() {
        let (mut vm, _registry) = make_vm();
        install_environment_source(
            &mut vm,
            MapEnvironmentSource::from_entries([("AIPO_TEST", "value")]),
        );
        grant_env(&mut vm);

        assert_eq!(
            get(&mut vm, &[name("AIPO_MISSING")]).expect("get succeeds"),
            Value::None
        );
        assert_eq!(
            has(&mut vm, &[name("AIPO_MISSING")]).expect("has succeeds"),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_environment_provider_is_read_only_and_revocable() {
        let (mut vm, _registry) = make_vm();
        install_environment_source(
            &mut vm,
            MapEnvironmentSource::from_entries([("KEY", "value")]),
        );
        assert_eq!(
            vm.host()
                .environment_source()
                .and_then(|source| source.get("KEY")),
            Some("value".to_string())
        );
        vm.host_context().revoke_environment();
        assert!(vm.host().environment_source().is_none());
    }

    #[test]
    fn test_env_providers_are_isolated_per_vm() {
        let (mut first, _first_registry) = make_vm();
        let (mut second, _second_registry) = make_vm();
        install_environment_source(
            &mut first,
            MapEnvironmentSource::from_entries([("SHARED", "first")]),
        );
        install_environment_source(
            &mut second,
            MapEnvironmentSource::from_entries([("SHARED", "second")]),
        );
        grant_env(&mut first);
        grant_env(&mut second);

        assert_eq!(
            get(&mut first, &[name("SHARED")]).expect("first get succeeds"),
            name("first")
        );
        assert_eq!(
            get(&mut second, &[name("SHARED")]).expect("second get succeeds"),
            name("second")
        );
    }

    #[test]
    fn test_env_metadata_declares_capability() {
        let (vm, registry) = make_vm();
        assert!(vm.get_global("env").is_some());
        let get_meta = registry
            .get(Some("env"), "get")
            .expect("env.get metadata is registered");
        let has_meta = registry
            .get(Some("env"), "has")
            .expect("env.has metadata is registered");
        assert_eq!(
            get_meta.capabilities,
            vec![Capability::ENV_READ.to_string()]
        );
        assert_eq!(
            has_meta.capabilities,
            vec![Capability::ENV_READ.to_string()]
        );
        assert!(
            registry
                .get(None, "len")
                .expect("prelude metadata is registered")
                .capabilities
                .is_empty()
        );
    }
}
