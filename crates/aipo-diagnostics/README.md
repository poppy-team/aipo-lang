# aipo-diagnostics

`aipo-diagnostics` defines the canonical diagnostic catalog, severity levels, structured error reporting, and output emitters (human-readable and JSON Lines) for the Aipo compiler.

## Architecture & Guarantees
- **Canonical Diagnostic Codes**: Stable `AIPO_LEX_*`, `AIPO_PARSE_*`, `AIPO_SEM_*`, `AIPO_IR_*` codes.
- **Machine & Human Emitters**: Supports `--message-format=jsonl` and ANSI colored terminal rendering with line/column context and source underline.
- **No Rust Panics**: All user-facing compilation errors are emitted as structured diagnostics.
- **Safety**: `#![forbid(unsafe_code)]`.
