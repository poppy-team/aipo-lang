# Wave 0 — Language MVP across 11 Slices

**Wave 0** established the foundational architecture of the Aipo programming language through 11 tightly integrated and thoroughly tested vertical slices.

---

## The 11 Slices Structure

### S1: Workspace, Source & Diagnostics (`aipo-source`, `aipo-diagnostics`)
- Rust workspace configuration using the 2024 edition.
- Canonical representation of source files (`Source`) with UTF-8 boundary safety (`is_char_boundary`).
- Structured diagnostic system with stable error codes, precise source spans, and human-readable error messages.

### S2: Lexer Core (`aipo-lexer`)
- Zero-copy, high-performance tokenizer for keywords, operators, and identifiers.
- Strict rejection of malformed numeric literals (`1.e5`, `1._5`), emitting `AIPO_LEX_INVALID_NUMBER`.

### S3: Parser Core & AST (`aipo-syntax`, `aipo-ast`)
- Recursive-descent parser featuring automatic panic-mode error recovery (`synchronize`).
- Fully typed Abstract Syntax Tree (AST) covering value expressions, struct definitions, functions, and top-level declarations.

### S4: HIR Lowering (`aipo-hir`)
- AST lowering into High-level Intermediate Representation (HIR).
- Desugaring of syntactic constructs and unified span attribution for structural hooks.

### S5: Semantic Analysis (`aipo-sema`)
- Lexical symbol tables supporting nested scopes and controlled variable shadowing.
- Static validation of `struct` declarations, duplicate field detection, and initial type checking.

### S6: Core IR & Bytecode (`aipo-ir`, `aipo-bytecode`)
- Linear intermediate representation optimized for explicit control flow.
- Deterministic bytecode compiler generating compact instructions for the virtual machine.

### S7: VM Core (`aipo-vm`)
- Stack- and register-aware bytecode interpreter for sequential execution.
- Call frame stack (`CallFrame`) maintaining strict isolation across local variables and evaluation temporaries.

### S8: Data & Errors (`aipo-vm`)
- Optimized `Value` representation.
- Structured fault subsystem (`VmFault`) seamlessly mapped to rich diagnostic reports.

### S9: Runtime & Minimal Stdlib (`aipo-runtime`, `aipo-stdlib`)
- Host runtime registration for native built-in functions.
- Core mathematical helpers, string manipulation, and standard output (`print`).

### S10: CLI & Formatter (`aipo-cli`, `aipo-formatter`)
- Unified command-line interface: `aipo run`, `aipo check`, and `aipo disasm`.
- Canonical automatic code formatter enforcing consistent indentation and spacing rules.

### S11: Initial Conformance Suite
- Baseline end-to-end integration test suite verifying that canonical test programs compile and run with deterministic, expected output.
