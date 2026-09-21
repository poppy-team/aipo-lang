//! `aipo-host` is the host ABI of Aipo: the contracts a host and a script share.
//!
//! The crate exists so the compiler never learns what a host *is*. A host describes its
//! surface as data ([`HostSchema`], the Aipo Host Schema), reaches its services only through
//! a deny-by-default capability set ([`CapabilitySet`]), and hands the script plain values
//! ([`HostValue`]) plus generational references ([`Handle`]) to objects it owns and can
//! release. Nothing in the IR or the bytecode names an engine, and no Rust reference or
//! lifetime crosses the boundary.
//!
//! Three rules from canon shape the whole crate
//! (`docs/canon/Aipo — Fechamento Arquitetural 10 10 …md` §11 and the Poppy Pivot host
//! profile section):
//!
//! 1. **Deny by default.** A profile grants nothing implicitly, and a declared capability set
//!    is an upper bound that policy may narrow, never widen ([`CapabilitySet::narrow`]).
//! 2. **Never use freed state.** A released handle resolves to stale rather than to whatever
//!    reuses its slot ([`HandleTable`]), and staleness is a fault, not a panic.
//! 3. **Host problems are faults.** A denied capability, a stale handle and a scoped binding
//!    that escapes all carry a stable diagnostic code ([`HostFault::code`]).
//!
//! # Example
//!
//! ```
//! use aipo_host::{Capability, CapabilitySet, HandleTable, HostValue};
//!
//! // Deny by default, then grant exactly one clock.
//! let mut granted = CapabilitySet::none();
//! granted.grant(Capability::parse("clock").expect("valid path"));
//! assert!(granted.require(&Capability::parse("clock.wall").expect("valid"), "time.now").is_ok());
//!
//! // A host object stays valid only while the host says so.
//! let mut objects = HandleTable::new();
//! let entity = objects.insert(HostValue::string("entity"));
//! assert!(objects.contains(entity));
//! objects.remove(entity);
//! assert!(objects.require(entity).is_err(), "stale handle faults");
//! ```
//!
//! # Status
//!
//! Slice `P03-G01`. The VM-side adapter (converting [`HostValue`] into a language value and
//! [`HostFault`] into a runtime fault) and the stdlib modules that use it land with the rest
//! of Wave 4; this crate is the contract they will be written against.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ahs;
pub mod capability;
pub mod fault;
pub mod handle;
pub mod value;

pub use ahs::{
    FieldSchema, FunctionSchema, HandleSchema, HostSchema, ModuleSchema, ParamSchema, TypeRef,
    TypeSchema, ValueSchema,
};
pub use capability::{CANON_CAPABILITIES, Capability, CapabilityError, CapabilitySet};
pub use fault::{HostFault, SchemaProblem};
pub use handle::{Handle, HandleTable};
pub use value::{HostValue, INT_MAX, INT_MIN};

#[cfg(test)]
mod tests {
    use super::*;

    /// The three rules of the crate, exercised through the public surface only.
    #[test]
    fn test_boundary_rules_hold_together() {
        // Rule 1: nothing is granted until a profile says so.
        let profile = CapabilitySet::none();
        assert!(
            profile
                .require(
                    &Capability::parse(Capability::POPPY).expect("valid"),
                    "world.spawn",
                )
                .is_err()
        );

        // Rule 2: a value the host sends is checked at the boundary, not at use.
        assert!(HostValue::int(INT_MAX + 1).is_err());
        assert!(HostValue::float(f64::NAN).is_err());

        // Rule 3: every refusal has a stable code.
        let mut objects: HandleTable<HostValue> = HandleTable::new();
        let handle = objects.insert(HostValue::Bool(true));
        objects.remove(handle);
        assert_eq!(
            objects.require(handle).expect_err("stale").code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_STALE_HANDLE
        );
    }
}
