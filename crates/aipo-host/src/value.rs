//! Host values: the only shapes that may cross the script boundary.
//!
//! Canon (Fechamento Arquitetural §11 and the Poppy Pivot host-profile section) says a host
//! value **copies by value** and external identity belongs to the host: no Rust reference,
//! lifetime or engine type is ever handed to a script. This module is that boundary in
//! type form — a closed enum of plain data plus a [`Handle`] when the script must refer to a
//! host-owned object.
//!
//! The constructors enforce the two numeric invariants the language already guarantees
//! everywhere else, so a host cannot smuggle an out-of-range `Int` or a non-finite `Float`
//! into a script: `Int` is limited to ±(2^53−1) and `Float` is finite-only. A violation is a
//! boundary fault, never a silent wrap.

use crate::fault::HostFault;
use crate::handle::Handle;

/// The largest magnitude an Aipo `Int` can hold, shared with the VM and the JS backend.
pub const INT_MAX: i64 = 9_007_199_254_740_991;

/// The smallest Aipo `Int`.
pub const INT_MIN: i64 = -INT_MAX;

/// A value crossing the host boundary.
///
/// Equality is structural over the transferred snapshot, which is what a host needs when it
/// compares what it sent with what came back. Language-level equality rules (`Float` zeros,
/// ordering, `same` versus `==`) belong to the VM's value layer and are deliberately not
/// encoded here.
#[derive(Debug, Clone, PartialEq)]
pub enum HostValue {
    /// `none`.
    None,
    /// A `Bool`.
    Bool(bool),
    /// An `Int`, always within ±(2^53−1).
    Int(i64),
    /// A finite `Float`.
    Float(f64),
    /// A `String` snapshot.
    ///
    /// NFC is enforced where the snapshot becomes a language value (the VM conversion
    /// point), not here: the lexer and the VM already own that invariant, and a second
    /// copy of it would be free to drift.
    String(String),
    /// A `Bytes` snapshot, copied by value.
    Bytes(Vec<u8>),
    /// A reference to a host-owned object, valid only until it is released.
    Handle(Handle),
}

impl HostValue {
    /// Builds an `Int`, refusing a value outside the language range.
    ///
    /// # Errors
    ///
    /// [`HostFault::InvalidHostValue`] when `value` leaves ±(2^53−1).
    pub fn int(value: i64) -> Result<Self, HostFault> {
        if !(INT_MIN..=INT_MAX).contains(&value) {
            return Err(HostFault::InvalidHostValue {
                detail: format!(
                    "Int {value} is outside ±{INT_MAX}; the language has no widening Int"
                ),
            });
        }
        Ok(Self::Int(value))
    }

    /// Builds a `Float`, refusing NaN and the infinities.
    ///
    /// # Errors
    ///
    /// [`HostFault::InvalidHostValue`] when `value` is not finite.
    pub fn float(value: f64) -> Result<Self, HostFault> {
        if !value.is_finite() {
            return Err(HostFault::InvalidHostValue {
                detail: format!("Float {value} is not finite; NaN and Infinity are faults"),
            });
        }
        Ok(Self::Float(value))
    }

    /// Builds a `String` snapshot. NFC is the VM boundary's job, see the variant docs.
    #[must_use]
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// The Aipo contract name a signature would write for this value.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Bool(_) => "Bool",
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::String(_) => "String",
            Self::Bytes(_) => "Bytes",
            Self::Handle(_) => "Handle",
        }
    }

    /// Whether the value is `none`.
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_within_range_is_accepted() {
        assert_eq!(HostValue::int(0), Ok(HostValue::Int(0)));
        assert_eq!(HostValue::int(INT_MAX), Ok(HostValue::Int(INT_MAX)));
        assert_eq!(HostValue::int(INT_MIN), Ok(HostValue::Int(INT_MIN)));
    }

    #[test]
    fn test_int_outside_range_is_a_fault() {
        let fault = HostValue::int(INT_MAX + 1).expect_err("out of range");
        assert_eq!(
            fault.code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_TYPE_MISMATCH
        );
        assert!(HostValue::int(i64::MAX).is_err());
    }

    #[test]
    fn test_non_finite_float_is_a_fault() {
        assert!(HostValue::float(f64::NAN).is_err());
        assert!(HostValue::float(f64::INFINITY).is_err());
        assert_eq!(HostValue::float(1.5), Ok(HostValue::Float(1.5)));
    }

    #[test]
    fn test_string_snapshot_copies_by_value() {
        assert_eq!(
            HostValue::string("\u{00e9}"),
            HostValue::String("\u{00e9}".to_string())
        );
    }

    #[test]
    fn test_type_names_match_the_written_contracts() {
        assert_eq!(HostValue::None.type_name(), "none");
        assert_eq!(HostValue::Bool(true).type_name(), "Bool");
        assert_eq!(HostValue::Bytes(vec![1]).type_name(), "Bytes");
    }

    #[test]
    fn test_a_host_value_is_plain_data() {
        // Copying a value must not carry identity: two equal snapshots are interchangeable.
        let original = HostValue::String("aipo".to_string());
        let copy = original.clone();
        assert_eq!(original, copy);
    }
}
