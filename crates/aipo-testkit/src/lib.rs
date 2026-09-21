//! Shared deterministic test harnesses for the Aipo workspace.
//!
//! Everything here is deterministic from an explicit seed: a failing case is
//! always reproducible from the seed printed by the harness. This crate is a
//! test-only utility (`publish = false`) and must never gain language semantics.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod corpus;
pub mod js;
pub mod meta;
pub mod pipeline;
pub mod proc;
pub mod rng;
pub mod smith;
pub mod timez;
