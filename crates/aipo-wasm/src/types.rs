//! WebAssembly type representations and mappings for Aipo (ADP-013).

use wasm_encoder::ValType;

/// Primitive WebAssembly value types supported by Aipo's Wasm backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    /// 32-bit integer (used for booleans, byte offsets, and flags).
    I32,
    /// 64-bit integer (used for Aipo 64-bit safe integers).
    I64,
    /// 64-bit IEEE 754 float (used for Aipo floats).
    F64,
}

impl From<WasmType> for ValType {
    fn from(t: WasmType) -> Self {
        match t {
            WasmType::I32 => ValType::I32,
            WasmType::I64 => ValType::I64,
            WasmType::F64 => ValType::F64,
        }
    }
}

/// Function signature in WebAssembly consisting of parameter and result types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmFnType {
    /// Parameter types.
    pub params: Vec<WasmType>,
    /// Result types.
    pub results: Vec<WasmType>,
}

impl WasmFnType {
    /// Creates a new function signature.
    #[must_use]
    pub fn new(params: Vec<WasmType>, results: Vec<WasmType>) -> Self {
        Self { params, results }
    }
}
