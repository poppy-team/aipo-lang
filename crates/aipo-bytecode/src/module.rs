//! Binary format and in-memory representation of compiled Aipo bytecode modules.

use crate::opcode::Constant;
use aipo_source::SourceSpan;
use serde::{Deserialize, Serialize};

/// Magic header bytes for Aipo Bytecode: `AIBC`.
pub const AIBC_MAGIC: [u8; 4] = *b"AIBC";
/// Current bytecode version.
pub const AIBC_VERSION: u16 = 1;

/// Metadata for one compiled function body in the module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// Function name; `__top_level__` for the module entry script and `Type.method`
    /// for `impl` methods.
    pub name: String,
    /// Absolute byte offset of the function entry point (before the slot prologue).
    pub entry_ip: usize,
    /// Number of declared parameters, which is also the expected call arity.
    pub params: usize,
    /// Number of declared locals after the parameters.
    pub locals: usize,
    /// `true` for `async fn`: calling produces a `Task` instead of running.
    pub is_async: bool,
}

/// A compiled, self-contained Aipo bytecode module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BytecodeModule {
    /// Magic header bytes (`AIBC`).
    pub magic: [u8; 4],
    /// Module format version.
    pub version: u16,
    /// Constant pool.
    pub constants: Vec<Constant>,
    /// Interned symbol/identifier names.
    pub names: Vec<String>,
    /// Bytecode instructions.
    pub code: Vec<u8>,
    /// Source span mapping for instruction byte offsets.
    pub spans: Vec<(usize, SourceSpan)>,
    /// Compiled function table, indexed by the operand of `MakeFunction`/`MakeClosure`.
    pub functions: Vec<FunctionInfo>,
    /// Declared struct layouts, used to materialize instances with canonical field names.
    pub structs: Vec<StructInfo>,
}

/// Layout metadata for one declared struct type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructInfo {
    /// Struct type name.
    pub name: String,
    /// Declared fields in canonical order: `(field name, is_fixed)`.
    pub fields: Vec<(String, bool)>,
}

impl BytecodeModule {
    /// Constructs a new empty bytecode module with valid magic and version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            magic: AIBC_MAGIC,
            version: AIBC_VERSION,
            constants: Vec::new(),
            names: Vec::new(),
            code: Vec::new(),
            spans: Vec::new(),
            functions: Vec::new(),
            structs: Vec::new(),
        }
    }

    /// Looks up a compiled function by its canonical name.
    #[must_use]
    pub fn function_by_name(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.iter().find(|f| f.name == name)
    }
}

impl Default for BytecodeModule {
    fn default() -> Self {
        Self::new()
    }
}
