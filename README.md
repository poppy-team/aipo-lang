# Aipo

> **TL;DR:** Aipo is a simple and fast programming language. It runs on a bytecode VM written
> in Rust, compiles to WebAssembly (Wasm) with high-performance JIT execution, and emits JavaScript.
> All execution backends produce identical, deterministic output.

```aipo
fn greet(name: String) -> String {
    return f"Hello, {name}!"
}

io.println(greet("world"))
```

## Start here (30 seconds)

You need: Rust 1.85+ and Node 20+.

1. Build the CLI:
   ```bash
   cargo build -p aipo-cli
   ```
2. Run a program:
   ```bash
   cargo run -q -p aipo-cli -- run examples/24_idiomatic_aipo_showcase.aipo
   ```
3. Run the test suite:
   ```bash
   cargo run -q -p aipo-cli -- test
   ```

## Developer Tooling & Commands

| Task | Command | Description |
|---|---|---|
| **Run Program** | `aipo run <file.aipo>` | Executes via VM or WebAssembly JIT (`--wasm`) |
| **Run Tests** | `aipo test [path] [--filter <pattern>]` | Automated test runner with temporal/PRNG isolation |
| **Type & Semantic Check** | `aipo check <file.aipo>` | Instant static analysis without execution |
| **Build Bundle / Wasm** | `aipo build <file.aipo> [--target <js\|wasm>]` | Emits JS bundle (`dist/app.js`) or Wasm binary (`dist/app.wasm`) |
| **Code Formatting** | `aipo fmt <files...> [--check]` | Opinionated canonical formatter (CI audit with `--check`) |
| **Package Management** | `aipo package <lock\|audit\|cache>` | Hermetic lockfiles, SHA pinning, and offline caching |
| **Bytecode / Wasm Disasm** | `aipo disasm <file.aibc\|file.wasm>` | Disassembles to readable bytecode or WAT text |

Exit codes: `0` = OK, `1` = Language or test failure, `2` = Command usage error.

Full reference: [`docs/tools/index.md`](docs/tools/index.md) and [`docs/reference/cli.md`](docs/reference/cli.md).

## What is Aipo?

- **Clean & Ergonomic Syntax**: Curly-brace blocks `{ ... }` without parentheses around conditions, designed for cognitive clarity and neurodivergent accessibility.
- **Strong Dynamic Typing with Optional Contracts**: Fast dynamic prototyping with structural signature contracts on boundaries (`fn compute(val: Int) -> Float`).
- **Immutability by Default**: Variables (`let`) and struct fields are immutable unless declared with `var`. Receiver mutation requires explicit `var self`.
- **Automatic Structural Interfaces**: Types satisfy interfaces automatically when signatures match, without ceremony or boilerplate.
- **Safe Unicode Strings**: Always valid UTF-8 with automatic NFC normalization, f-strings (`f"Hello, {name}!"`), raw strings (`r"..."`), and multi-line strings (`"""..."""`).
- **Two Separate Error Channels**:
  - Recoverable failures: `fail(msg)`, captured via `attempt { ... } failed err { ... }` with automatic mutation rollback, or fallback with `or_else`.
  - Programming faults: Fatal violations (overflow, contract breach) halt execution deterministically.
- **Cooperative Virtual-Time Async**: `async fn`, sequential `await do { ... }`, and structured concurrency combinators (`task.spawn`, `task.all`, `task.race`, `task.sleep`).
- **Standard Library Built-in**: Global modules (`math`, `string`, `io`, `task`, `time`, `random`, `json`, `binary`, `path`, `url`, `regex`, `expect`, `testing`, `fs`, `env`).
- **Dual Execution Substrates**: Native WebAssembly (Wasm 2.0 / WASI) with JIT execution via Wasmtime, Stack VM interpreter, and optimized ES2022 JavaScript emitter.
- **Hermetic Package Manager**: Cryptographically pinned dependencies by commit SHA, deterministic lockfiles, and immutable local cache.

## Where things live

| Path | What it is |
|---|---|
| `crates/` | The implementation (compiler, VM, stdlib, CLI, JS backend) |
| `docs/conformance/` | The test corpus — the language referee |
| `docs/canon/` | The language rules (what wins arguments) |
| `docs/waves/` | Plans per delivery wave |
| `docs/evidence/` | Proof that each goal was done |
| `docs/reference/cli.md` | Command manual |
| `examples/` | Sample programs |

## Contributing and governance

**To contribute, read [`CONTRIBUTING.md`](CONTRIBUTING.md) first.** The short rules:

1. No code without an explicit Goal.
2. Never guess language semantics — open questions become written decisions (ADPs).
3. Keep all quality gates green (see below).
4. Update docs together with code.

This project is governed by **Prumo v0.6** (human-agent collaboration with lean
context). The local entry points are:

- [`prumo.json`](prumo.json) — project manifest.
- [`ENTRYPOINT.md`](ENTRYPOINT.md) — context router.
- [`PROJECT_STATE.md`](PROJECT_STATE.md) — current state.
- [`docs/PRUMO.md`](docs/PRUMO.md) — documentation map.

> Prumo framework: [github.com/raillen/prumo](https://github.com/raillen/prumo).

## Quality gates

Run all of these. All must be green.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
prumo validate . && prumo doctor .
```

## License

Licensed under the [MIT License](LICENSE).
