//! VM-side adapter for the host ABI (`aipo-host`).
//!
//! `aipo-host` owns the contracts a host and a script share; this module is the one place
//! that consumes them, so the value model, the fault model and the capability gate have a
//! single conversion point instead of each call site inventing its own.
//!
//! Three conversions live here:
//!
//! - **Values.** A [`HostValue`] becomes a [`Value`] and back. Host values are plain data, so
//!   the conversion never hands a script a Rust reference; a `String` is normalized to NFC as
//!   it becomes a language value, because NFC belongs to the language's `String` boundary and
//!   the host side deliberately does not duplicate that rule.
//! - **Faults.** A [`HostFault`] becomes a [`VmFault`], never a panic and never a recoverable
//!   `Failure`: canon classifies a denied capability, a stale handle and an escaped binding as
//!   programming errors at the boundary.
//! - **Capabilities.** [`HostContext`] holds the deny-by-default [`CapabilitySet`] the running
//!   profile granted plus the host objects behind their handles. A privileged operation calls
//!   [`HostContext::require`] first, so a missing capability is always the same fault.

use std::cell::RefCell;
use std::rc::Rc;

use aipo_host::{Capability, CapabilitySet, Handle, HandleTable, HostFault, HostValue};
use unicode_normalization::UnicodeNormalization;

use crate::fault::VmFault;
use crate::value::{Value, check_finite_float, check_safe_int};

/// Converts a host value into a language value.
///
/// # Errors
///
/// [`VmFault`] when the snapshot cannot be a language value: an `Int` outside ±(2^53−1), a
/// non-finite `Float`, or a handle the boundary does not model.
pub fn host_value_to_value(host: &HostValue) -> Result<Value, VmFault> {
    match host {
        HostValue::None => Ok(Value::None),
        HostValue::Bool(value) => Ok(Value::Bool(*value)),
        HostValue::Int(value) => check_safe_int(*value).map(Value::Int),
        HostValue::Float(value) => check_finite_float(*value).map(Value::Float),
        // The host side keeps `HostValue::String` a raw snapshot (see its docs), so NFC is
        // applied here, where a snapshot becomes a language `String`.
        HostValue::String(text) => Ok(Value::String(Rc::new(text.nfc().collect()))),
        HostValue::Bytes(bytes) => Ok(Value::Bytes(Rc::new(RefCell::new(bytes.clone())))),
        HostValue::Handle(handle) => Ok(Value::HostHandle(*handle)),
    }
}

/// Converts a language value into a host value, when the value may cross the boundary.
///
/// Returns `None` for a value the host ABI does not model (a function, a struct instance, a
/// collection), because canon says a host receives plain data plus handles: anything richer
/// must be described by the AHS instead of pushed through by reference.
#[must_use]
pub fn value_to_host_value(value: &Value) -> Option<HostValue> {
    match value {
        Value::None | Value::Unset => Some(HostValue::None),
        Value::Bool(inner) => Some(HostValue::Bool(*inner)),
        Value::Int(inner) => Some(HostValue::Int(*inner)),
        Value::Float(inner) => Some(HostValue::Float(*inner)),
        Value::String(inner) => Some(HostValue::String(inner.to_string())),
        Value::Bytes(inner) => Some(HostValue::Bytes(inner.borrow().clone())),
        Value::HostHandle(handle) => Some(HostValue::Handle(*handle)),
        _ => None,
    }
}

/// Maps a host-boundary fault onto the VM's fault model.
///
/// The mapping is total and one-way: the VM never turns a fault back into a recoverable
/// `Failure`, because canon puts every one of these in the fault column.
#[must_use]
pub fn host_fault_to_vm_fault(fault: &HostFault) -> VmFault {
    match fault {
        HostFault::CapabilityDenied {
            capability,
            operation,
        } => VmFault::CapabilityDenied {
            capability: capability.name().to_string(),
            operation: operation.clone(),
        },
        HostFault::StaleHandle { handle } => VmFault::StaleHandle {
            handle: handle.to_string(),
        },
        HostFault::ScopeEscape { binding } => VmFault::ScopeEscape {
            binding: binding.clone(),
        },
        // A host value that cannot satisfy its contract and a rejected host surface are both
        // boundary type problems, which the VM already reports as a type mismatch.
        HostFault::InvalidHostValue { detail } => VmFault::TypeMismatch {
            expected: "host value satisfying its declared contract".to_string(),
            actual: detail.clone(),
        },
        HostFault::Schema { problems } => VmFault::TypeMismatch {
            expected: "a consistent host surface description".to_string(),
            actual: problems
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        },
    }
}

/// The host services a running VM may reach.
///
/// Holds what the profile granted and the host objects those services hand out. A host object
/// is reachable only through a [`Handle`], and the table is the only thing that resolves one,
/// so a script cannot address host memory directly.
#[derive(Debug, Default)]
pub struct HostContext {
    granted: CapabilitySet,
    objects: HandleTable<HostValue>,
}

impl HostContext {
    /// A context that grants nothing: canon's default for a sandboxed host.
    #[must_use]
    pub fn denied() -> Self {
        Self::default()
    }

    /// A context with exactly the given capabilities granted.
    #[must_use]
    pub fn with_capabilities(granted: CapabilitySet) -> Self {
        Self {
            granted,
            objects: HandleTable::new(),
        }
    }

    /// The capabilities this profile granted.
    #[must_use]
    pub fn granted(&self) -> &CapabilitySet {
        &self.granted
    }

    /// Grants one more capability to the running profile (host-side only).
    pub fn grant(&mut self, capability: Capability) -> bool {
        self.granted.grant(capability)
    }

    /// Whether a privileged operation is allowed.
    #[must_use]
    pub fn allows(&self, capability: &Capability) -> bool {
        self.granted.allows(capability)
    }

    /// Checks a capability, mapping the denial onto the VM fault model.
    ///
    /// # Errors
    ///
    /// [`VmFault::CapabilityDenied`] naming the capability and the operation.
    pub fn require(&self, capability: &Capability, operation: &str) -> Result<(), VmFault> {
        self.granted
            .require(capability, operation)
            .map_err(|fault| host_fault_to_vm_fault(&fault))
    }

    /// Gives the script ownership of a host value and returns its handle.
    pub fn hand_out(&mut self, value: HostValue) -> Handle {
        self.objects.insert(value)
    }

    /// The value behind a handle, or `None` when the handle is stale.
    #[must_use]
    pub fn lookup(&self, handle: Handle) -> Option<&HostValue> {
        self.objects.get(handle)
    }

    /// The value behind a handle, or the stale-handle fault.
    ///
    /// # Errors
    ///
    /// [`VmFault::StaleHandle`] when the handle was released or belongs to another generation.
    pub fn resolve(&self, handle: Handle) -> Result<&HostValue, VmFault> {
        self.objects
            .require(handle)
            .map_err(|fault| host_fault_to_vm_fault(&fault))
    }

    /// Releases a host object, invalidating every handle minted for its slot.
    pub fn release(&mut self, handle: Handle) -> Option<HostValue> {
        self.objects.remove(handle)
    }

    /// Number of live host objects, for diagnostics and tests.
    #[must_use]
    pub fn live_objects(&self) -> usize {
        self.objects.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability(path: &str) -> Capability {
        Capability::parse(path).expect("test capability is valid")
    }

    #[test]
    fn test_host_values_round_trip() {
        for host in [
            HostValue::None,
            HostValue::Bool(true),
            HostValue::Int(42),
            HostValue::Float(1.5),
            HostValue::string("aipo"),
            HostValue::Bytes(vec![1, 2, 3]),
        ] {
            let value = host_value_to_value(&host).expect("converts");
            let back = value_to_host_value(&value).expect("converts back");
            assert_eq!(back, host);
        }
    }

    #[test]
    fn test_out_of_range_host_value_is_a_fault_not_a_wrap() {
        // The `aipo-host` constructors already refuse these, so a value this wrong can only
        // arrive from a host that bypassed them; the VM must still refuse to wrap it.
        let fault = host_value_to_value(&HostValue::Int(i64::MAX)).expect_err("out of range");
        assert_eq!(
            fault.diagnostic_code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_OVERFLOW
        );
    }

    #[test]
    fn test_string_snapshot_is_normalized_at_this_boundary() {
        // `e` + combining acute becomes the precomposed character, NFC being the language's
        // String boundary rule.
        let decomposed = "e\u{0301}".to_string();
        let value = host_value_to_value(&HostValue::String(decomposed)).expect("converts");
        assert_eq!(value, Value::String(Rc::new("\u{00e9}".to_string())));
    }

    #[test]
    fn test_richer_values_do_not_cross_by_reference() {
        // A collection has no host-value form: the AHS describes it instead, so a host never
        // receives script memory by reference.
        let list = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
        assert_eq!(value_to_host_value(&list), None);
    }

    #[test]
    fn test_denied_capability_is_a_fault_with_the_canon_code() {
        let context = HostContext::denied();
        let fault = context
            .require(&capability("clock.wall"), "time.now")
            .expect_err("denied by default");
        assert_eq!(
            fault.diagnostic_code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
        );
        assert!(fault.to_string().contains("time.now"));
    }

    #[test]
    fn test_granting_an_ancestor_allows_the_leaf() {
        let mut context = HostContext::denied();
        context.grant(capability("clock"));
        assert!(
            context
                .require(&capability("clock.monotonic"), "time.monotonic")
                .is_ok()
        );
        assert!(
            context
                .require(&capability("clock.wall"), "time.now")
                .is_ok()
        );
    }

    #[test]
    fn test_stale_handle_is_a_fault_and_the_holder_is_unchanged() {
        let mut context = HostContext::denied();
        let handle = context.hand_out(HostValue::string("entity"));
        assert_eq!(context.live_objects(), 1);
        context.release(handle);
        assert_eq!(context.live_objects(), 0);
        let fault = context.resolve(handle).expect_err("stale");
        assert_eq!(
            fault.diagnostic_code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_STALE_HANDLE
        );
    }

    #[test]
    fn test_host_faults_map_onto_the_vm_fault_model() {
        let cases = [
            (
                HostFault::CapabilityDenied {
                    capability: capability("filesystem.read"),
                    operation: "fs.read".to_string(),
                },
                aipo_diagnostics::DiagnosticCode::AIPO_RT_CAPABILITY_DENIED,
            ),
            (
                HostFault::ScopeEscape {
                    binding: "held".to_string(),
                },
                aipo_diagnostics::DiagnosticCode::AIPO_RT_SCOPE_ESCAPE,
            ),
            (
                HostFault::InvalidHostValue {
                    detail: "not a number".to_string(),
                },
                aipo_diagnostics::DiagnosticCode::AIPO_RT_TYPE_MISMATCH,
            ),
        ];
        for (fault, expected) in cases {
            assert_eq!(host_fault_to_vm_fault(&fault).diagnostic_code(), expected);
        }
    }
}
