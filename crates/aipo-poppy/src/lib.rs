//! `aipo-poppy` — Poppy Game Engine reference profile and deterministic headless adapter for Aipo.
//!
//! This crate implements the Poppy host profile over `aipo-host`:
//! - Describes the Poppy game engine surface as data via AHS ([`poppy_schema`]).
//! - Manages game entities through generational [`aipo_host::Handle`]s, preventing use-after-free.
//! - Defers structural ECS mutations into a [`CommandBuffer`] applied strictly at safe points.
//! - Drives deterministic headless game simulation with fixed ticks and seeded PRNG.
//! - Adapts Poppy native operations into the Aipo VM environment behind `poppy.*` capabilities.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod adapter;
pub mod commands;
pub mod prng;
pub mod schema;
pub mod simulation;
pub mod world;

pub use adapter::{
    POPPY_LOCK, PoppyService, create_module, install_poppy, register_poppy, revoke_poppy,
};
pub use commands::{CommandBuffer, PoppyCommand};
pub use prng::PoppyRng;
pub use schema::poppy_schema;
pub use simulation::Simulation;
pub use world::{EntityRecord, World};
