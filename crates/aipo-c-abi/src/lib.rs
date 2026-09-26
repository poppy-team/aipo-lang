//! Synchronous, versioned, single-threaded C ABI and safe Rust embedding interface for Aipo (ADP-008, ADP-009).

pub mod c_api;
pub mod runtime;
pub mod types;

pub use c_api::*;
pub use runtime::*;
pub use types::*;
