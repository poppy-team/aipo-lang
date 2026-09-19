//! SourceMap registry managing loaded source files.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::SourceError;
use crate::source::{Source, SourceId};

/// Manages multiple loaded source files and provides lookup by ID and name.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SourceMap {
    sources: Vec<Source>,
    by_name: HashMap<String, SourceId>,
}

impl SourceMap {
    /// Creates a new empty `SourceMap`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    /// Adds in-memory source text to the registry.
    pub fn add_source(&mut self, name: impl Into<String>, content: &str) -> SourceId {
        let name = name.into();
        let id = SourceId::next();
        let source = Source::new(id, name.clone(), content);

        self.sources.push(source);
        self.by_name.insert(name, id);
        id
    }

    /// Loads a source file from disk, validating UTF-8 and normalizing newlines.
    pub fn load_file(&mut self, path: &Path) -> Result<SourceId, SourceError> {
        let path_str = path.to_string_lossy().into_owned();

        let raw_bytes = fs::read(path).map_err(|err| SourceError::Io {
            path: path_str.clone(),
            detail: err.to_string(),
        })?;

        let raw_text = std::str::from_utf8(&raw_bytes).map_err(|err| SourceError::InvalidUtf8 {
            path: path_str.clone(),
            detail: err.to_string(),
        })?;

        Ok(self.add_source(path_str, raw_text))
    }

    /// Retrieves a reference to a source file by its unique `SourceId`.
    #[must_use]
    pub fn get(&self, id: SourceId) -> Option<&Source> {
        self.sources.iter().find(|s| s.id() == id)
    }

    /// Retrieves a reference to a source file by its name or path.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&Source> {
        let id = self.by_name.get(name)?;
        self.get(*id)
    }

    /// Returns an iterator over all registered sources.
    pub fn iter(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter()
    }

    /// Returns the total number of registered sources.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Returns true if the source map contains no sources.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}
