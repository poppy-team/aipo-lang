//! Host-boundary faults.
//!
//! Canon is explicit that a host-side problem is a *fault*, not a panic and not a
//! recoverable `Failure`: a denied capability, a stale handle and a scoped binding that
//! escapes are all programming errors at the boundary
//! (`docs/canon/Aipo — Fechamento Arquitetural 10 10 …md` §16, "Host stale handle/security
//! fault não aparece como panic Rust"). Every variant therefore carries the context a
//! diagnostic needs and maps to one stable [`DiagnosticCode`].

use std::fmt;

use aipo_diagnostics::DiagnosticCode;

use crate::capability::Capability;
use crate::handle::Handle;

/// Something the host boundary refused to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostFault {
    /// The operation needs a capability the running profile does not grant.
    CapabilityDenied {
        /// The capability that was required.
        capability: Capability,
        /// The host operation that required it, as the script would name it.
        operation: String,
    },
    /// A handle addressed a slot that was released or reused.
    StaleHandle {
        /// The handle that no longer addresses a live value.
        handle: Handle,
    },
    /// A binding created inside a scoped callback reached a heap-publication point outside
    /// the scope it belongs to.
    ScopeEscape {
        /// The handle that tried to escape.
        handle: Handle,
        /// The binding name or publication site the host knows it by.
        binding: String,
    },
    /// A value offered at the boundary cannot satisfy the contract it is crossing.
    InvalidHostValue {
        /// What was wrong with the value.
        detail: String,
    },
    /// A host surface description is internally inconsistent.
    Schema {
        /// The problems found, each with its path inside the description.
        problems: Vec<SchemaProblem>,
    },
}

/// One inconsistency inside a host surface description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaProblem {
    /// Where the problem is, as a dotted path (`module.function(param)`).
    pub path: String,
    /// What is wrong.
    pub message: String,
}

impl SchemaProblem {
    /// Builds a problem report.
    #[must_use]
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for SchemaProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl HostFault {
    /// The stable diagnostic code for this fault.
    ///
    /// Canon names `AIPO_RT_CAPABILITY_DENIED`; the other two codes are implementation
    /// detail for restrictions canon mandates without naming a code, which is the same
    /// latitude ADP-006 §F recorded for the await diagnostics.
    #[must_use]
    pub fn code(&self) -> DiagnosticCode {
        match self {
            Self::CapabilityDenied { .. } => DiagnosticCode::AIPO_RT_CAPABILITY_DENIED,
            Self::StaleHandle { .. } => DiagnosticCode::AIPO_RT_STALE_HANDLE,
            Self::ScopeEscape { .. } => DiagnosticCode::AIPO_RT_SCOPE_ESCAPE,
            // A host value that cannot satisfy its declared contract is a contract fault at
            // a boundary, which canon classifies as a type mismatch.
            Self::InvalidHostValue { .. } => DiagnosticCode::AIPO_RT_TYPE_MISMATCH,
            // A rejected host description is a malformed input to the boundary.
            Self::Schema { .. } => DiagnosticCode::AIPO_RT_INVALID_SCHEMA,
        }
    }

    /// The human-readable message, carrying the target/capability context canon asks for.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::CapabilityDenied {
                capability,
                operation,
            } => format!(
                "operation '{operation}' requires capability '{capability}', which this host does not grant"
            ),
            Self::StaleHandle { handle } => format!(
                "{handle} is stale: the host object it referenced was released, so it may not be used"
            ),
            Self::ScopeEscape { handle, binding } => format!(
                "{handle} (binding '{binding}') was created in a scoped host callback and cannot leave it"
            ),
            Self::InvalidHostValue { detail } => {
                format!("host value does not satisfy its contract: {detail}")
            }
            Self::Schema { problems } => {
                let detail = problems
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("host surface description is invalid: {detail}")
            }
        }
    }
}

impl fmt::Display for HostFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for HostFault {}

#[cfg(test)]
mod tests {
    use super::*;

    /// A handle that addresses a real slot, for the tests that only need the identity.
    fn sample_handle() -> Handle {
        let mut table: crate::HandleTable<u8> = crate::HandleTable::new();
        table.insert(1)
    }

    #[test]
    fn test_capability_denial_carries_operation_and_capability() {
        let fault = HostFault::CapabilityDenied {
            capability: Capability::parse("clock.wall").expect("valid"),
            operation: "time.now".to_string(),
        };
        assert_eq!(fault.code(), DiagnosticCode::AIPO_RT_CAPABILITY_DENIED);
        let message = fault.message();
        assert!(message.contains("time.now"), "{message}");
        assert!(message.contains("clock.wall"), "{message}");
    }

    #[test]
    fn test_display_includes_the_code() {
        let fault = HostFault::ScopeEscape {
            handle: sample_handle(),
            binding: "held".to_string(),
        };
        let rendered = fault.to_string();
        assert!(rendered.starts_with("AIPO_RT_SCOPE_ESCAPE"), "{rendered}");
    }

    #[test]
    fn test_schema_fault_lists_every_problem() {
        let fault = HostFault::Schema {
            problems: vec![
                SchemaProblem::new("host", "name is empty"),
                SchemaProblem::new("mod.f", "duplicate"),
            ],
        };
        let message = fault.message();
        assert!(message.contains("host: name is empty"), "{message}");
        assert!(message.contains("mod.f: duplicate"), "{message}");
    }
}
