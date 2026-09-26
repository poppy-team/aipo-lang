//! WebAssembly binary module emitter using `wasm-encoder` (ADP-013).

use crate::types::WasmFnType;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Module, TypeSection, ValType,
};

/// High-level builder and binary emitter for standard WebAssembly modules.
#[derive(Default)]
pub struct WasmEmitter {
    types: Vec<WasmFnType>,
    function_types: Vec<u32>,
    exports: Vec<(String, ExportKind, u32)>,
    code: Vec<Function>,
}

impl WasmEmitter {
    /// Creates a new, empty WebAssembly module emitter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a function signature and returns its type index.
    pub fn add_type(&mut self, fn_type: WasmFnType) -> u32 {
        if let Some(idx) = self.types.iter().position(|t| t == &fn_type) {
            return idx as u32;
        }
        let idx = self.types.len() as u32;
        self.types.push(fn_type);
        idx
    }

    /// Adds a compiled function with its type index and code body, returning the function index.
    pub fn add_function(&mut self, type_index: u32, body: Function) -> u32 {
        let fn_idx = self.function_types.len() as u32;
        self.function_types.push(type_index);
        self.code.push(body);
        fn_idx
    }

    /// Exports a function under a public symbol name.
    pub fn export_function(&mut self, name: impl Into<String>, func_index: u32) {
        self.exports
            .push((name.into(), ExportKind::Func, func_index));
    }

    /// Finalizes the WebAssembly module and returns its encoded binary representation (`.wasm`).
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        let mut module = Module::new();

        // 1. Type Section
        if !self.types.is_empty() {
            let mut type_section = TypeSection::new();
            for fn_type in &self.types {
                let params: Vec<ValType> = fn_type.params.iter().copied().map(Into::into).collect();
                let results: Vec<ValType> =
                    fn_type.results.iter().copied().map(Into::into).collect();
                type_section.ty().function(params, results);
            }
            module.section(&type_section);
        }

        // 2. Function Section
        if !self.function_types.is_empty() {
            let mut fn_section = FunctionSection::new();
            for &type_idx in &self.function_types {
                fn_section.function(type_idx);
            }
            module.section(&fn_section);
        }

        // 3. Export Section
        if !self.exports.is_empty() {
            let mut export_section = ExportSection::new();
            for (name, kind, index) in &self.exports {
                export_section.export(name, *kind, *index);
            }
            module.section(&export_section);
        }

        // 4. Code Section
        if !self.code.is_empty() {
            let mut code_section = CodeSection::new();
            for func in &self.code {
                code_section.function(func);
            }
            module.section(&code_section);
        }

        module.finish()
    }
}
