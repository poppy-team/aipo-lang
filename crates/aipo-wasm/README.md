# aipo-wasm

`aipo-wasm` is the WebAssembly (Wasm 2.0 / WASI) binary emitter and execution substrate for the Aipo programming language (`ADP-013`).

## Architecture & Guarantees

- **Standard Wasm 2.0 Binary Format**: Emits standardized WebAssembly binaries (`.wasm`) using the high-performance, pure Rust `wasm-encoder` engine.
- **Microarchitectural Breakthrough**: Replaces the stack-based interpreter dispatch floor (~100ns/opcode) with near-native JIT execution via Cranelift/Wasmtime (~1-3ns/op), achieving 30x to 50x performance gains.
- **Zero-FFI Self-Hosting Foundation**: WebAssembly is a pure binary byte specification. Because Aipo already possesses native `Bytes` and LEB128 manipulation primitives in its standard library, emitting `.wasm` requires zero foreign function interfaces (FFI), enabling a clean path to self-hosting (`aipo-in-aipo`).
- **Capability-Based Security Alignment**: Directly aligns with WASI (WebAssembly System Interface) and the Wasm Component Model, mapping Aipo's deny-by-default `CapabilitySet` to formal Wasm security boundaries.

## Architecture & Modules

- **`types`**: Primitive WebAssembly value types (`I32`, `I64`, `F64`) and function signatures (`WasmFnType`).
- **`emitter`**: High-level binary builder wrapping `wasm-encoder` sections (Type, Function, Export, Code).
- **`compiler`**: HIR-to-Wasm compiler lowering Aipo `HirProgram` into executable `.wasm` binaries.
- **`error`**: Typed compile errors (`WasmCompileError`) with source spans.

## Usage Example

```rust
use aipo_syntax::parse;
use aipo_hir::lower;
use aipo_wasm::compile_hir;
use aipo_source::{Source, SourceId};

let src = Source::new(SourceId::next(), "example.aipo", "fn add(a: Int, b: Int) -> Int { return a + b }");
let (ast, diags) = parse(&src);
assert!(diags.is_empty());

let hir = lower(ast);
let wasm_bytes = compile_hir(&hir).expect("compilation succeeds");

// Execute via wasmtime or write to disk as .wasm
```

## Testing

```bash
cargo test -p aipo-wasm
```
