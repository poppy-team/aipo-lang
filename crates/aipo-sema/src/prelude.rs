//! Description of the globally visible language surface.

use crate::symbol::SymbolKind;
use std::collections::HashMap;

/// The set of names that are visible at the root scope before any user declaration.
///
/// The analyzer itself knows only the *language* surface: the fundamental values
/// (`none`, `true`, `false`) and the core type values (`Int`, `Float`, `Byte`, `String`,
/// `Bool`, `List`, `Dict`, `Bytes`, `Range`). Everything the standard library adds is
/// supplied by the caller through [`crate::check_with_prelude`], so `aipo-sema` never has
/// to depend on `aipo-stdlib` and the two can never drift silently: the composition root
/// (the CLI) derives the surface from the *same* registration the VM executes with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreludeSurface {
    globals: HashMap<String, SymbolKind>,
}

impl PreludeSurface {
    /// Creates an empty surface.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the surface of the Aipo language itself, without the standard library.
    #[must_use]
    pub fn fundamental() -> Self {
        let mut surface = Self::new();
        for name in ["none", "true", "false"] {
            surface.add_variable(name);
        }
        for name in [
            "Int", "Float", "Byte", "String", "Bool", "List", "Dict", "Bytes", "Range",
        ] {
            surface.add_variable(name);
        }
        surface
    }

    /// Adds a variable-like or type-value global.
    pub fn add_variable(&mut self, name: &str) {
        self.globals.insert(name.to_string(), SymbolKind::Variable);
    }

    /// Adds a global function with its accepted argument counts.
    pub fn add_function(&mut self, name: &str, min_args: usize, max_args: usize) {
        self.globals.insert(
            name.to_string(),
            SymbolKind::Function { min_args, max_args },
        );
    }

    /// Returns whether `name` is part of the surface.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.globals.contains_key(name)
    }

    /// Iterates over the surface entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolKind)> {
        self.globals.iter()
    }

    /// Number of names in the surface.
    #[must_use]
    pub fn len(&self) -> usize {
        self.globals.len()
    }

    /// Returns whether the surface is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.globals.is_empty()
    }
}
