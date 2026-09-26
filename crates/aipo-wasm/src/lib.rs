//! WebAssembly (Wasm 2.0 / WASI) binary module emitter and execution foundation for Aipo (ADP-013).

pub mod emitter;
pub mod types;

pub use emitter::WasmEmitter;
pub use types::{WasmFnType, WasmType};
pub use wasm_encoder::Instruction;
