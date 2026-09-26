//! WebAssembly binary module emitter using `wasm-encoder` (ADP-013).

use crate::types::WasmFnType;
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ElementSection, Elements, EntityType, ExportKind,
    ExportSection, Function, FunctionSection, GlobalSection, GlobalType, ImportSection,
    MemorySection, MemoryType, Module, RefType, TableSection, TableType, TypeSection, ValType,
};

/// High-level builder and binary emitter for standard WebAssembly modules.
#[derive(Default)]
pub struct WasmEmitter {
    types: Vec<WasmFnType>,
    imports: Vec<(String, String, EntityType)>,
    num_imported_funcs: u32,
    function_types: Vec<u32>,
    exports: Vec<(String, ExportKind, u32)>,
    code: Vec<Function>,
    table: Option<TableType>,
    elements: Vec<(u32, i32, Vec<u32>)>,
    memory: Option<MemoryType>,
    globals: Vec<(GlobalType, ConstExpr)>,
    data_segments: Vec<(u32, i32, Vec<u8>)>,
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

    /// Registers an imported function and returns its assigned function index.
    pub fn add_import_func(&mut self, module: &str, name: &str, type_index: u32) -> u32 {
        let fn_idx = self.num_imported_funcs;
        self.num_imported_funcs += 1;
        self.imports.push((
            module.to_string(),
            name.to_string(),
            EntityType::Function(type_index),
        ));
        fn_idx
    }

    /// Returns the number of imported functions.
    #[must_use]
    pub fn num_imported_funcs(&self) -> u32 {
        self.num_imported_funcs
    }

    /// Returns the function index that will be assigned to the next function added.
    #[must_use]
    pub fn next_func_idx(&self) -> u32 {
        self.num_imported_funcs + self.function_types.len() as u32
    }

    /// Adds a compiled function with its type index and code body, returning the function index.
    pub fn add_function(&mut self, type_index: u32, body: Function) -> u32 {
        let fn_idx = self.num_imported_funcs + self.function_types.len() as u32;
        self.function_types.push(type_index);
        self.code.push(body);
        fn_idx
    }

    /// Exports a function under a public symbol name.
    pub fn export_function(&mut self, name: impl Into<String>, func_index: u32) {
        self.exports
            .push((name.into(), ExportKind::Func, func_index));
    }

    /// Enables the indirect function table (Table 0) with funcref elements.
    pub fn enable_table(&mut self, minimum: u64, maximum: Option<u64>) {
        self.table = Some(TableType {
            element_type: RefType::FUNCREF,
            minimum,
            maximum,
            table64: false,
            shared: false,
        });
    }

    /// Adds an element segment targeting a table at a fixed offset with function indices.
    pub fn add_element_segment(&mut self, table_index: u32, offset: i32, func_indices: Vec<u32>) {
        self.elements.push((table_index, offset, func_indices));
    }

    /// Enables linear memory with initial and optional maximum pages (each page is 64 KiB).
    pub fn enable_memory(&mut self, initial_pages: u64, max_pages: Option<u64>) {
        self.memory = Some(MemoryType {
            minimum: initial_pages,
            maximum: max_pages,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
    }

    /// Exports the linear memory under the given public symbol (usually "memory").
    pub fn export_memory(&mut self, name: impl Into<String>) {
        self.exports.push((name.into(), ExportKind::Memory, 0));
    }

    /// Exports a global variable under the given public symbol.
    pub fn export_global(&mut self, name: impl Into<String>, global_index: u32) {
        self.exports
            .push((name.into(), ExportKind::Global, global_index));
    }

    /// Adds a global variable and returns its global index.
    pub fn add_global(&mut self, global_type: GlobalType, init_expr: &ConstExpr) -> u32 {
        let idx = self.globals.len() as u32;
        self.globals.push((global_type, init_expr.clone()));
        idx
    }

    /// Adds an active data segment into the data section targeting memory at the given byte offset.
    pub fn add_data_segment(&mut self, memory_index: u32, offset: i32, data: Vec<u8>) {
        self.data_segments.push((memory_index, offset, data));
    }

    /// Finalizes the WebAssembly module and returns its encoded binary representation (`.wasm`).
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        let mut module = Module::new();

        // 1. Type Section (1)
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

        // 1.5. Import Section (2)
        if !self.imports.is_empty() {
            let mut import_section = ImportSection::new();
            for (module_name, name, entity) in &self.imports {
                import_section.import(module_name, name, *entity);
            }
            module.section(&import_section);
        }

        // 2. Function Section (3)
        if !self.function_types.is_empty() {
            let mut fn_section = FunctionSection::new();
            for &type_idx in &self.function_types {
                fn_section.function(type_idx);
            }
            module.section(&fn_section);
        }

        // 3. Table Section (4)
        if let Some(table_type) = self.table {
            let mut table_section = TableSection::new();
            table_section.table(table_type);
            module.section(&table_section);
        }

        // 4. Memory Section (5)
        if let Some(mem_type) = self.memory {
            let mut mem_section = MemorySection::new();
            mem_section.memory(mem_type);
            module.section(&mem_section);
        }

        // 5. Global Section (6)
        if !self.globals.is_empty() {
            let mut global_section = GlobalSection::new();
            for (gt, init) in &self.globals {
                global_section.global(*gt, init);
            }
            module.section(&global_section);
        }

        // 6. Export Section (7)
        if !self.exports.is_empty() {
            let mut export_section = ExportSection::new();
            for (name, kind, index) in &self.exports {
                export_section.export(name, *kind, *index);
            }
            module.section(&export_section);
        }

        // 7. Element Section (9)
        if !self.elements.is_empty() {
            let mut elem_section = ElementSection::new();
            for (table_idx, offset, func_indices) in &self.elements {
                elem_section.active(
                    Some(*table_idx),
                    &ConstExpr::i32_const(*offset),
                    Elements::Functions(std::borrow::Cow::Borrowed(func_indices.as_slice())),
                );
            }
            module.section(&elem_section);
        }

        // 8. Code Section (10)
        if !self.code.is_empty() {
            let mut code_section = CodeSection::new();
            for func in &self.code {
                code_section.function(func);
            }
            module.section(&code_section);
        }

        // 9. Data Section (11)
        if !self.data_segments.is_empty() {
            let mut data_section = DataSection::new();
            for (mem_idx, offset, bytes) in &self.data_segments {
                data_section.active(
                    *mem_idx,
                    &ConstExpr::i32_const(*offset),
                    bytes.iter().copied(),
                );
            }
            module.section(&data_section);
        }

        module.finish()
    }
}
