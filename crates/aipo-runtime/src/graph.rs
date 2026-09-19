//! Dependency graph analysis, acyclicity verification, and deterministic initialization ordering.

use crate::error::RuntimeError;
use crate::module::ModuleRecord;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Directed dependency graph of runtime modules.
#[derive(Debug, Default, Clone)]
pub struct ModuleGraph {
    modules: HashMap<String, ModuleRecord>,
}

impl ModuleGraph {
    /// Creates an empty module graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a module record into the graph.
    pub fn register(&mut self, record: ModuleRecord) {
        self.modules.insert(record.path.clone(), record);
    }

    /// Retrieves a reference to a registered module record by its canonical path.
    #[must_use]
    pub fn get(&self, path: &str) -> Option<&ModuleRecord> {
        self.modules.get(path)
    }

    /// Retrieves a mutable reference to a registered module record.
    pub fn get_mut(&mut self, path: &str) -> Option<&mut ModuleRecord> {
        self.modules.get_mut(path)
    }

    /// Returns the count of registered modules.
    #[must_use]
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Checks if the module graph is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Computes the deterministic, topological initialization order of all registered modules.
    ///
    /// Dependencies precede their dependents. Ties are broken lexicographically by canonical path.
    ///
    /// # Errors
    /// Returns `RuntimeError::ModuleNotFound` if a module references an unregistered dependency,
    /// or `RuntimeError::CyclicDependency` if a circular import dependency is detected.
    pub fn topological_init_order(&self) -> Result<Vec<String>, RuntimeError> {
        // Validate that all dependencies exist
        for (path, record) in &self.modules {
            for dep in &record.dependencies {
                if !self.modules.contains_key(dep) {
                    return Err(RuntimeError::ModuleNotFound { name: dep.clone() });
                }
            }
            let _ = path;
        }

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        let mut path_stack = Vec::new();

        // Sort module keys for deterministic DFS start
        let mut sorted_keys: Vec<_> = self.modules.keys().cloned().collect();
        sorted_keys.sort();

        for key in &sorted_keys {
            if !visited.contains(key) {
                self.detect_cycles_dfs(key, &mut visited, &mut in_stack, &mut path_stack)?;
            }
        }

        // Kahn's algorithm for deterministic topological order
        // In-degree for module M = number of dependencies not yet initialized
        let mut dep_count: BTreeMap<String, usize> = BTreeMap::new();
        let mut dependents_of: HashMap<String, Vec<String>> = HashMap::new();

        for (path, record) in &self.modules {
            dep_count.insert(path.clone(), record.dependencies.len());
            for dep in &record.dependencies {
                dependents_of
                    .entry(dep.clone())
                    .or_default()
                    .push(path.clone());
            }
        }

        let mut ready: Vec<String> = dep_count
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(path, _)| path.clone())
            .collect();
        ready.sort(); // Lexicographical tie-break

        let mut order = Vec::with_capacity(self.modules.len());

        while !ready.is_empty() {
            // Pop the smallest canonical path (first element of sorted vec)
            let curr = ready.remove(0);
            order.push(curr.clone());

            if let Some(dependents) = dependents_of.get(&curr) {
                for dep in dependents {
                    if let Some(count) = dep_count.get_mut(dep) {
                        *count -= 1;
                        if *count == 0 {
                            ready.push(dep.clone());
                            ready.sort(); // Re-sort for tie-break
                        }
                    }
                }
            }
        }

        if order.len() != self.modules.len() {
            // Fallback cycle detection
            return Err(RuntimeError::CyclicDependency {
                cycle: vec!["unresolved dependency cycle".to_string()],
            });
        }

        Ok(order)
    }

    fn detect_cycles_dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        path_stack: &mut Vec<String>,
    ) -> Result<(), RuntimeError> {
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());
        path_stack.push(node.to_string());

        if let Some(record) = self.modules.get(node) {
            let mut sorted_deps = record.dependencies.clone();
            sorted_deps.sort();

            for dep in &sorted_deps {
                if !visited.contains(dep) {
                    self.detect_cycles_dfs(dep, visited, in_stack, path_stack)?;
                } else if in_stack.contains(dep) {
                    // Extract cycle chain
                    let start_idx = path_stack.iter().position(|p| p == dep).unwrap_or(0);
                    let mut cycle: Vec<String> = path_stack[start_idx..].to_vec();
                    cycle.push(dep.clone());
                    return Err(RuntimeError::CyclicDependency { cycle });
                }
            }
        }

        in_stack.remove(node);
        path_stack.pop();
        Ok(())
    }
}
