//! Runtime error taxonomy and diagnostics mapping for modules and registry.

use aipo_diagnostics::DiagnosticCode;
use std::fmt;

/// Errors arising during module resolution, dependency cycle checks, or initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// A requested module cannot be located in the module registry.
    ModuleNotFound {
        /// Canonical path of the requested module.
        name: String,
    },
    /// A circular import dependency chain was detected.
    CyclicDependency {
        /// Ordered sequence of module paths forming the cycle.
        cycle: Vec<String>,
    },
    /// A requested export is missing from the target module.
    ExportNotFound {
        /// Module canonical path.
        module: String,
        /// Name of the missing exported symbol.
        symbol: String,
    },
    /// Module initialization failed during execution.
    InitializationFailed {
        /// Module canonical path.
        module: String,
        /// Failure reason.
        reason: String,
    },
}

impl RuntimeError {
    /// Maps runtime errors to stable diagnostic codes.
    #[must_use]
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::ModuleNotFound { .. } => DiagnosticCode::AIPO_SEM_UNKNOWN_MODULE,
            Self::CyclicDependency { .. } => DiagnosticCode::AIPO_SEM_IMPORT_CYCLE,
            Self::ExportNotFound { .. } => DiagnosticCode::AIPO_SEM_EXPORT_UNKNOWN,
            Self::InitializationFailed { .. } => DiagnosticCode::AIPO_RT_MODULE_INIT_FAILED,
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModuleNotFound { name } => {
                write!(
                    f,
                    "runtime error [AIPO_SEM_UNKNOWN_MODULE]: module '{name}' could not be resolved"
                )
            }
            Self::CyclicDependency { cycle } => {
                write!(
                    f,
                    "runtime error [AIPO_SEM_IMPORT_CYCLE]: circular module dependency: {}",
                    cycle.join(" -> ")
                )
            }
            Self::ExportNotFound { module, symbol } => {
                write!(
                    f,
                    "runtime error [AIPO_SEM_EXPORT_UNKNOWN]: module '{module}' has no export '{symbol}'"
                )
            }
            Self::InitializationFailed { module, reason } => {
                write!(
                    f,
                    "runtime error [AIPO_RT_MODULE_INIT_FAILED]: initialization of module '{module}' failed: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
