//! Native function and built-in module registration catalog.

use std::collections::HashMap;

/// Metadata describing a native function or method provided by the runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeFunctionMeta {
    /// Function identifier.
    pub name: String,
    /// Expected parameter count.
    pub arity: usize,
    /// Parent module path (`None` for Prelude functions).
    pub module: Option<String>,
    /// Brief documentation of purpose.
    pub doc: String,
}

impl NativeFunctionMeta {
    /// Creates new native function metadata.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        arity: usize,
        module: Option<&str>,
        doc: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            arity,
            module: module.map(ToString::to_string),
            doc: doc.into(),
        }
    }
}

/// Catalog of native functions registered for prelude and built-in modules.
#[derive(Debug, Default, Clone)]
pub struct NativeRegistry {
    functions: HashMap<(Option<String>, String), NativeFunctionMeta>,
}

impl NativeRegistry {
    /// Constructs a new empty native registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a native function.
    pub fn register(&mut self, meta: NativeFunctionMeta) {
        let key = (meta.module.clone(), meta.name.clone());
        self.functions.insert(key, meta);
    }

    /// Looks up a native function by optional module path and name.
    #[must_use]
    pub fn get(&self, module: Option<&str>, name: &str) -> Option<&NativeFunctionMeta> {
        let key = (module.map(ToString::to_string), name.to_string());
        self.functions.get(&key)
    }

    /// Lists all native functions belonging to the global Prelude.
    #[must_use]
    pub fn list_prelude(&self) -> Vec<&NativeFunctionMeta> {
        self.functions
            .iter()
            .filter(|((m, _), _)| m.is_none())
            .map(|(_, meta)| meta)
            .collect()
    }

    /// Lists all native functions belonging to a specific built-in module.
    #[must_use]
    pub fn list_module(&self, module: &str) -> Vec<&NativeFunctionMeta> {
        self.functions
            .iter()
            .filter(|((m, _), _)| m.as_deref() == Some(module))
            .map(|(_, meta)| meta)
            .collect()
    }

    /// Returns a list of distinct built-in module names registered.
    #[must_use]
    pub fn list_modules(&self) -> Vec<String> {
        let mut mods: Vec<_> = self
            .functions
            .keys()
            .filter_map(|(m, _)| m.clone())
            .collect();
        mods.sort();
        mods.dedup();
        mods
    }
}
