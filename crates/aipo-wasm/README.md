# aipo-wasm

`aipo-wasm` is the WebAssembly (Wasm 2.0 / WASI) binary emitter and execution substrate for the Aipo programming language (`ADP-013`).

## Architecture & Guarantees

- **Standard Wasm 2.0 Binary Format**: Emits standardized WebAssembly binaries (`.wasm`) using the high-performance, pure Rust `wasm-encoder` engine.
- **Microarchitectural Breakthrough**: Replaces the stack-based interpreter dispatch floor (~100ns/opcode) with near-native JIT execution via Cranelift/Wasmtime (~1-3ns/op), achieving 30x to 50x performance gains.
- **Zero-FFI Self-Hosting Foundation**: WebAssembly is a pure binary byte specification. Because Aipo already possesses native `Bytes` and LEB128 manipulation primitives in its standard library, emitting `.wasm` requires zero foreign function interfaces (FFI), enabling a clean path to self-hosting (`aipo-in-aipo`).
- **Capability-Based Security Alignment**: Directly aligns with WASI (WebAssembly System Interface) and the Wasm Component Model, mapping Aipo's deny-by-default `CapabilitySet` to formal Wasm security boundaries.

## Testing

```bash
cargo test -p aipo-wasm
```
