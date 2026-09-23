//! Module representation and lifecycle state in the Aipo runtime.

use aipo_bytecode::BytecodeModule;
use std::collections::HashMap;

/// Lifecycle state of a module during a runtime execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleState {
    /// Discovered or registered, awaiting initialization.
    Uninitialized,
    /// Currently undergoing top-level evaluation (used for cycle detection).
    Initializing,
    /// Successfully evaluated; exports are finalized.
    Initialized,
    /// Failed evaluation; published nothing.
    Failed(String),
}

/// A loaded module record within the runtime graph.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleRecord {
    /// Canonical module path (e.g. `game.player` or `math`).
    pub path: String,
    /// Canonical paths of dependencies imported by this module.
    pub dependencies: Vec<String>,
    /// Compiled bytecode image for the module, if compiled from Aipo source.
    pub bytecode: Option<BytecodeModule>,
    /// Current lifecycle state.
    pub state: ModuleState,
    /// Exported symbol map.
    pub exports: HashMap<String, String>,
}

impl ModuleRecord {
    /// Creates a new uninitialized module record.
    ///
    /// The path is used verbatim as the graph key; [`crate::ModuleGraph::register`]
    /// rejects empty, untrimmed or duplicate paths, so callers that build records by
    /// hand still get the deterministic canonical form enforced at registration.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        dependencies: Vec<String>,
        bytecode: Option<BytecodeModule>,
    ) -> Self {
        Self {
            path: path.into(),
            dependencies,
            bytecode,
            state: ModuleState::Uninitialized,
            exports: HashMap::new(),
        }
    }

    /// Checks if the module has successfully completed initialization.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        matches!(self.state, ModuleState::Initialized)
    }
}
