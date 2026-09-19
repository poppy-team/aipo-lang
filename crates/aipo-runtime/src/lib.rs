//! Shared runtime services for Aipo: module registry, dependency graph,
//! deterministic initialization ordering, and native function catalog.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod graph;
pub mod module;
pub mod native;

pub use error::RuntimeError;
pub use graph::ModuleGraph;
pub use module::{ModuleRecord, ModuleState};
pub use native::{NativeFunctionMeta, NativeRegistry};
