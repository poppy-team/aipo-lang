# aipo-bytecode

`aipo-bytecode` defines the compact instruction set, constant pool representation, binary module format (`AIBC`), bytecode emitter, structural verifier, and disassembler for the Aipo compiler.

## Architecture & Guarantees
- **Instruction Set (`OpCode`)**: Compact stack-based opcode set covering arithmetic, variables, collections, field access, calls, and jumps.
- **Binary Module Format (`AIBC`)**: Versioned header (`AIBC` v1), indexed constant pool, symbol names table, and byte-encoded instructions with source span mapping.
- **Structural Verifier**: Rigorous pre-load validation verifying instruction bounds, constant pool indices, name indices, and valid instruction-aligned jump targets.
- **Disassembler**: Human-readable disassembly listing format for debugging and deterministic snapshot testing.
- **Safety**: `#![forbid(unsafe_code)]`.
