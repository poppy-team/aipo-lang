# aipo-cli

`aipo-cli` implements the command-line interface and orchestrator for the Aipo programming language (`aipo`).

## Command Surface

```text
aipo run <path> [--wasm] [--package-cache <dir>] [--message-format=<human|jsonl>]
aipo test [path] [--filter <pattern>] [--package-cache <dir>] [--message-format=<human|jsonl>]
aipo check <path> [--wasm] [--package-cache <dir>] [--message-format=<human|jsonl>]
aipo build <path> [--target <js|wasm>] [--out <dir>] [--package-cache <dir>] [--message-format=<human|jsonl>]
aipo disasm <path> [--wasm] [--package-cache <dir>] [--message-format=<human|jsonl>]
aipo fmt <paths...> [--check]
aipo package lock <package-dir> [--fetch-github --cache <dir>] [--github-token-env <name>]
aipo package audit <package-dir>
aipo package cache verify <cache-dir>
aipo package cache prune <cache-dir> --lock <lockfile> [--apply]
aipo --version
aipo --help
```

## Commands

- **`run`**: Executes Aipo source files (`.aipo`), bytecode files (`.aibc`), or WebAssembly binaries (`.wasm`). Passing `--wasm` or `-t wasm` compiles `.aipo` directly to WebAssembly and runs it via the Wasmtime JIT engine.
- **`test`**: Automatically discovers and executes isolated unit test suites (`*_test.aipo`, `test_*.aipo`).
- **`check`**: Validates lexing, parsing, and semantic analysis without executing.
- **`build`**: Compiles an Aipo program into distributable bundles:
  - `--target js` (default): Emits JavaScript bundle (`app.js`, `aipo-runtime.js`, `app.js.map`).
  - `--target wasm` / `--wasm`: Emits a standalone WebAssembly binary (`dist/app.wasm`).
- **`disasm`**: Prints human-readable disassembly listings:
  - `.aibc` or `.aipo`: Bytecode disassembly with source mappings.
  - `.wasm` or `.aipo --wasm`: WebAssembly Text format (WAT).
- **`fmt`**: Canonical source code formatter; `--check` detects drift without modifying files.
- **`package`**: Dependency resolution, lockfile creation, security auditing, and package cache management.

## Exit Codes

- `0` — Success.
- `1` — Language or package failure (diagnostic emitted, runtime fault, or formatting drift).
- `2` — Usage error (invalid flag, missing argument, unreadable file).

## Guarantees

- **`#![forbid(unsafe_code)]`**: Strictly memory safe Rust.
- **Pure Injectable I/O**: `run_with(&args, out, err)` enables deterministic, process-free testing.
