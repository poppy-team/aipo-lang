//! Diagnostic severity levels.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A fatal or non-recoverable compile-time error.
    Error,
    /// A runtime fault (e.g. integer overflow, division by zero).
    Fault,
    /// A non-fatal warning about suspicious or discouraged constructs.
    Warning,
    /// An informational note providing additional context.
    Note,
}

impl Severity {
    /// Returns the lowercase string identifier for this severity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Fault => "fault",
            Self::Warning => "warning",
            Self::Note => "note",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}
