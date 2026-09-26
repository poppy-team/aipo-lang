//! Safe runtime embedding context for Aipo (ADP-008, ADP-009).

use crate::types::{aipo_handle_t, aipo_host_fn_t, aipo_status_t, aipo_value_t};
use aipo_bytecode::BytecodeModule;
use aipo_diagnostics::Severity;
use aipo_host::{Capability, CapabilitySet, Handle, HostValue};
use aipo_sema::PreludeSurface;
use aipo_source::{Source, SourceId};
use aipo_vm::{Value, Vm, VmError, VmFault};
use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};

thread_local! {
    static CURRENT_RUNTIME: Cell<*mut AipoRuntime> = const { Cell::new(std::ptr::null_mut()) };
}

struct CurrentRuntimeGuard(*mut AipoRuntime);

impl CurrentRuntimeGuard {
    fn new(rt: *mut AipoRuntime) -> Self {
        let prev = CURRENT_RUNTIME.get();
        CURRENT_RUNTIME.set(rt);
        Self(prev)
    }
}

impl Drop for CurrentRuntimeGuard {
    fn drop(&mut self) {
        CURRENT_RUNTIME.set(self.0);
    }
}

/// Registration metadata for a host-provided C function.
#[derive(Clone, Copy)]
pub struct HostFnEntry {
    /// Expected arity.
    pub arity: usize,
    /// Function pointer to C callback.
    pub callback: aipo_host_fn_t,
}

/// Safe, self-contained Aipo runtime for host embedders.
pub struct AipoRuntime {
    /// Internal VM instance.
    pub vm: Vm,
    /// Granted capabilities for this runtime instance.
    pub capabilities: CapabilitySet,
    /// Loaded bytecode modules by module name.
    pub modules: HashMap<String, BytecodeModule>,
    /// Last error or fault message recorded.
    pub last_error: Option<String>,
    /// Pool of null-terminated C strings to ensure memory remains valid across C calls.
    pub string_pool: Vec<CString>,
    /// Registered host functions by name.
    pub host_functions: HashMap<String, (HostFnEntry, Option<String>)>,
}

impl Default for AipoRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AipoRuntime {
    /// Creates a new, isolated runtime with deny-by-default capabilities.
    #[must_use]
    pub fn new() -> Self {
        let mut vm = Vm::new();
        let mut registry = aipo_runtime::NativeRegistry::new();
        aipo_stdlib::register_stdlib(&mut vm, &mut registry);

        Self {
            vm,
            capabilities: CapabilitySet::none(),
            modules: HashMap::new(),
            last_error: None,
            string_pool: Vec::new(),
            host_functions: HashMap::new(),
        }
    }

    /// Allocates and pins a string in the pool, returning its C pointer.
    pub fn pool_string(&mut self, s: &str) -> *const std::ffi::c_char {
        let c_str = CString::new(s).unwrap_or_else(|_| CString::new("<invalid string>").unwrap());
        let ptr = c_str.as_ptr();
        self.string_pool.push(c_str);
        ptr
    }

    /// Grants a host capability path (e.g. `"clock"`, `"io"`, `"poppy"`).
    pub fn grant(&mut self, capability_name: &str) -> Result<(), String> {
        let cap = Capability::parse(capability_name).map_err(|e| e.to_string())?;
        self.capabilities.grant(cap);
        Ok(())
    }

    /// Revokes a host capability path.
    pub fn revoke(&mut self, capability_name: &str) -> Result<(), String> {
        let cap = Capability::parse(capability_name).map_err(|e| e.to_string())?;
        self.capabilities.revoke(&cap);
        Ok(())
    }

    /// Compiles and loads an Aipo module from source text.
    pub fn load_module(
        &mut self,
        name: &str,
        source_text: &str,
    ) -> Result<(), (aipo_status_t, String)> {
        let source = Source::new(SourceId::next(), name, source_text);
        let (program, diagnostics) = aipo_syntax::parse(&source);
        if diagnostics.iter().any(|d| d.severity == Severity::Error) {
            let msg = format!("parse error in module '{name}'");
            self.last_error = Some(msg.clone());
            return Err((aipo_status_t::AIPO_ERR_DIAGNOSTIC, msg));
        }

        let hir = aipo_hir::lower(program);
        let mut surface = PreludeSurface::fundamental();
        for global in self.vm.globals.keys() {
            surface.add_variable(global);
        }
        for mod_name in self.modules.keys() {
            surface.add_variable(mod_name);
        }

        let (_, sema_diagnostics) = aipo_sema::check_with_prelude(&source, &hir, &surface);
        if sema_diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
        {
            let msg = format!("semantic error in module '{name}'");
            self.last_error = Some(msg.clone());
            return Err((aipo_status_t::AIPO_ERR_DIAGNOSTIC, msg));
        }

        let ir = aipo_ir::lower_to_ir(&hir);
        let module = aipo_bytecode::compile(&ir).map_err(|errors| {
            let msg = format!("compilation error: {:?}", errors);
            (aipo_status_t::AIPO_ERR_DIAGNOSTIC, msg)
        })?;

        // Register structs and methods on VM
        for decl in &module.structs {
            let fields: Vec<(&str, bool)> = decl
                .fields
                .iter()
                .map(|(name, fixed)| (name.as_str(), *fixed))
                .collect();
            self.vm.register_struct(decl.name.clone(), fields);
        }
        for function in &module.functions {
            if let Some((type_name, method)) = function.name.split_once('.') {
                self.vm.register_struct_method(
                    type_name,
                    method,
                    function.entry_ip,
                    function.params,
                    function.is_async,
                );
            }
        }

        // Execute top-level module code
        let _guard = CurrentRuntimeGuard::new(self as *mut AipoRuntime);
        if let Err(err) = self.vm.run(&module) {
            let msg = format!("runtime error initializing module '{name}': {err}");
            self.last_error = Some(msg.clone());
            let status = match err {
                VmError::UncaughtFailure(_) => aipo_status_t::AIPO_ERR_UNCAUGHT_FAILURE,
                _ => aipo_status_t::AIPO_ERR_FAULT,
            };
            return Err((status, msg));
        }

        self.modules.insert(name.to_string(), module);
        Ok(())
    }

    /// Invokes a global function in a loaded module or global scope.
    pub fn call(
        &mut self,
        module_name: &str,
        func_name: &str,
        args: &[aipo_value_t],
    ) -> Result<aipo_value_t, (aipo_status_t, String)> {
        let module = self.modules.get(module_name).cloned().ok_or_else(|| {
            let msg = format!("module '{module_name}' is not loaded");
            self.last_error = Some(msg.clone());
            (aipo_status_t::AIPO_ERR_USAGE, msg)
        })?;

        // Resolve function value
        let callee = self
            .vm
            .globals
            .get(func_name)
            .cloned()
            .or_else(|| {
                module
                    .functions
                    .iter()
                    .find(|f| f.name == func_name)
                    .map(|f| Value::Function {
                        entry_ip: f.entry_ip as u32,
                        arity: f.params as u16,
                        is_async: f.is_async,
                    })
            })
            .ok_or_else(|| {
                let msg = format!("function '{func_name}' not found in module '{module_name}'");
                self.last_error = Some(msg.clone());
                (aipo_status_t::AIPO_ERR_USAGE, msg)
            })?;

        let mut vm_args = Vec::with_capacity(args.len());
        for arg in args {
            let v = arg.to_vm_value().map_err(|e| {
                self.last_error = Some(e.clone());
                (aipo_status_t::AIPO_ERR_USAGE, e)
            })?;
            vm_args.push(v);
        }

        let _guard = CurrentRuntimeGuard::new(self as *mut AipoRuntime);
        let result = self.vm.invoke(&module, callee, &vm_args).map_err(|err| {
            let msg = format!("runtime error calling '{func_name}': {err}");
            self.last_error = Some(msg.clone());
            let status = match err {
                VmError::UncaughtFailure(_) => aipo_status_t::AIPO_ERR_UNCAUGHT_FAILURE,
                _ => aipo_status_t::AIPO_ERR_FAULT,
            };
            (status, msg)
        })?;

        let c_val = aipo_value_t::from_vm_value(&result, |s| self.pool_string(s));
        Ok(c_val)
    }

    /// Registers a host native function implemented in C.
    pub fn register_host_fn(
        &mut self,
        name: &str,
        arity: usize,
        required_capability: Option<&str>,
        callback: aipo_host_fn_t,
    ) {
        let entry = HostFnEntry { arity, callback };
        self.host_functions.insert(
            name.to_string(),
            (entry, required_capability.map(str::to_string)),
        );

        // Register dummy native on VM that will be routed via host_natives
        self.vm
            .define_global(name, Value::native(name, arity, dummy_native_fn));
        self.vm
            .register_host_native(name, arity, host_native_dispatcher);
    }

    /// Creates a generational handle for a host value.
    pub fn handle_create(&mut self, val: HostValue) -> aipo_handle_t {
        let handle = self.vm.host_context().hand_out(val);
        handle.into()
    }

    /// Resolves a live generational handle to its host value.
    pub fn handle_resolve(&self, handle: aipo_handle_t) -> Result<HostValue, VmFault> {
        let h: Handle = handle.into();
        self.vm.host().resolve(h).cloned()
    }

    /// Releases a generational handle, invalidating it forever.
    pub fn handle_release(&mut self, handle: aipo_handle_t) -> Result<(), VmFault> {
        let h: Handle = handle.into();
        self.vm
            .host_context()
            .release(h)
            .map(|_| ())
            .ok_or_else(|| VmFault::StaleHandle {
                handle: h.to_string(),
            })
    }
}

fn dummy_native_fn(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::None)
}

fn host_native_dispatcher(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let rt_ptr = CURRENT_RUNTIME.get();
    if rt_ptr.is_null() {
        return Err(VmFault::TypeMismatch {
            expected: "active runtime context".to_string(),
            actual: "null".to_string(),
        }
        .into());
    }

    let rt = unsafe { &mut *rt_ptr };

    // Resolve function name from the callee on the VM stack
    let callee_name = vm
        .stack
        .len()
        .checked_sub(1 + args.len())
        .and_then(|idx| vm.stack.get(idx))
        .and_then(|val| match val {
            Value::Native(data) => Some(data.name.clone()),
            _ => None,
        });

    let (name, entry, required_cap) = if let Some(ref c_name) = callee_name {
        rt.host_functions
            .get(c_name)
            .map(|(e, cap)| (c_name.clone(), *e, cap.clone()))
            .or_else(|| {
                rt.host_functions
                    .iter()
                    .find(|(_, (e, _))| e.arity == args.len())
                    .map(|(n, (e, cap))| (n.clone(), *e, cap.clone()))
            })
    } else {
        rt.host_functions
            .iter()
            .find(|(_, (e, _))| e.arity == args.len())
            .map(|(n, (e, cap))| (n.clone(), *e, cap.clone()))
    }
    .ok_or_else(|| {
        VmError::from(VmFault::NotCallable {
            type_name: "unregistered host function".to_string(),
        })
    })?;

    // Capability check
    if let Some(ref cap_name) = required_cap {
        let cap = Capability::parse(cap_name).map_err(|e| VmFault::CapabilityDenied {
            capability: cap_name.clone(),
            operation: e.to_string(),
        })?;
        if let Err(e) = rt.capabilities.require(&cap, &name) {
            return Err(VmFault::CapabilityDenied {
                capability: cap_name.clone(),
                operation: e.to_string(),
            }
            .into());
        }
    }

    // Convert arguments to aipo_value_t
    let c_args: Vec<aipo_value_t> = args
        .iter()
        .map(|v| aipo_value_t::from_vm_value(v, |s| rt.pool_string(s)))
        .collect();

    let mut out_result = aipo_value_t::none();
    let status = (entry.callback)(
        rt_ptr as *mut crate::types::aipo_runtime_t,
        c_args.as_ptr(),
        c_args.len(),
        &mut out_result,
    );

    match status {
        aipo_status_t::AIPO_OK => {
            let val = out_result
                .to_vm_value()
                .map_err(|e| VmFault::TypeMismatch {
                    expected: "valid return value".to_string(),
                    actual: e,
                })?;
            Ok(val)
        }
        aipo_status_t::AIPO_ERR_UNCAUGHT_FAILURE => {
            let msg = if out_result.str_ptr.is_null() {
                "host failure".to_string()
            } else {
                unsafe { CStr::from_ptr(out_result.str_ptr) }
                    .to_str()
                    .unwrap_or("host failure")
                    .to_string()
            };
            Ok(Value::failure(msg))
        }
        aipo_status_t::AIPO_ERR_CAPABILITY_DENIED => Err(VmFault::CapabilityDenied {
            capability: required_cap.unwrap_or_default(),
            operation: name.to_string(),
        }
        .into()),
        aipo_status_t::AIPO_ERR_STALE_HANDLE => Err(VmFault::StaleHandle {
            handle: Handle::from(out_result.handle_val).to_string(),
        }
        .into()),
        _ => Err(VmFault::TypeMismatch {
            expected: "AIPO_OK".to_string(),
            actual: format!("{status:?}"),
        }
        .into()),
    }
}
