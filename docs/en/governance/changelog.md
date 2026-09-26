# Official Project Changelog

All notable changes, releases, and milestones of the Aipo programming language are documented here, adhering to [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased — Towards v0.1.0]

### Product & Scoping
- **Scoping `aipo v0.1.0` (ADP-008)**: First official release is strictly language-first, delivering cooperative async, unified CLI, minimalist test runner, versioned synchronous C ABI (ADP-009), and thin interop proofs across Rust, C, and JS (ADP-010).

### Performance & Empirical Rigor
- **Cross-Language Benchmark Suite**: `aipo-bench --compare` test harness covering 6 deterministic workloads measured against Lua, LuaJIT, Wren, Luau, CPython, PyPy, Ruby, and native Rust.
- **Monomorphic Field Slot Cache**: Accelerated `GetField`/`SetField` dispatch yielding a 28.31% wall-clock reduction on the `fields` workload.
- **Base Frame Pointer Caching**: Consistent speedups across `arithmetic` (-3.71%), `fields` (-6.65%), and `recursion` (-4.17%).
- **Elimination of Clones in `SetField`**: Deterministically elided 600,000 `Value` clones on unguarded struct mutations.
- **Transparent Post-Mortem on Reverted Experiments**: Documented 7 micro-optimization experiments reverted due to lack of measurable gain, standardizing CPU process time measurement and closing the interpreter micro-tuning track.

### Hermetic Package Manager & Supply Chain (P04-G01 to P04-G11)
- **Manifest & Lockfile**: `namespace.package` coordinates, `aipo.toml` manifest, and canonical `aipo.lock`.
- **Remote Dependencies**: GitHub dependency resolution strictly requiring pinned 40-character commit SHAs.
- **Local Cache & Auditing**: Subcommands `aipo package fetch-github`, `aipo package cache verify`, and `aipo package cache prune --apply`.
- **Secure Authentication**: Support for environment variable Bearer tokens (`--github-token-env`), never writing secrets to disk or logs.

### Bytecode & Disassembly
- **`.aibc` Binary Serialization**: Deterministic binary format with magic header `AIBC` v1 verification.
- **Direct Execution**: `aipo run app.aibc` executes compiled bytecode without frontend compilation overhead.
- **Disassembler `aipo disasm`**: Precise mapping of bytecode instructions to source lines and columns.

---

## [Wave 4] — Host ABI & Poppy Simulation
- `aipo-host` crate with deny-by-default capability tree (`CapabilitySet`).
- Generational handles (`HandleTable`) immune to use-after-free hazards.
- Scope escape prevention across all 6 VM publication points.
- `aipo-poppy` crate featuring deterministic headless ECS simulation and command buffering.

---

## [Wave 3] — Rich Types & Cooperative Async
- `Set` with insertion ordering, lazy `Sequence`, and `Bytes` packing.
- `async fn` syntax and `await do ... end` block scoping.
- Cooperative scheduler with deterministic virtual time and `task.*` combinators.
- Runtime task dependency cycle detection (`AIPO_RT_AWAIT_CYCLE`).

---

## [Wave 2] — JavaScript Parity & Deep Quality
- `aipo-js` semantic compiler emitting ES2022 and Source Maps V3.
- Differential VM ↔ Node.js conformance test suite.
- Continuous fuzzing with libFuzzer and property testing with proptest.
- Automated dependency auditing with `cargo deny`.

---

## [Wave 1] — Structural Contracts & Transactions
- `init()` and `invariant()` lifecycle hooks.
- Automatic transactional mutation rollback inside `attempt ... recover`.
- Structural interfaces and method contract compliance.
- Nested local functions with self-recursion support (`FillSelfCapture`).

---

## [Wave 0] — Language Foundation (MVP)
- Initial implementation of the 11 vertical slices (Lexer, Parser, HIR, SEMA, IR, Bytecode, VM, Stdlib, Formatter, CLI, Conformance).
