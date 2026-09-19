# aipo-ir

`aipo-ir` provides the target-neutral Core Intermediate Representation (Core IR) for the Aipo compiler.

## Architecture & Guarantees
- **Target Neutrality**: Clean representation independent of bytecode or JavaScript backends.
- **Constant Pool Ready**: Converts literal tokens into typed constants (`Int(i64)`, `Float(f64)`, `String`, `Bool`, `None`).
- **Sequential Instruction Model**: Flat instruction sequences per function (`CoreFunction`) and top-level script (`__top_level__`).
- **Safety**: `#![forbid(unsafe_code)]` with zero runtime panics.
