# Aipo — Project Scope

**Status:** normative
**Scope:** in-scope capabilities, non-goals, compatibility constraints
**Blocking question:** What is explicitly out of scope?

## In-scope capabilities (Wave 1 MVP + Wave 2 slice W2-1)

- Fundamentals: Literals (`none`, `true`/`false`, Int, Float, Byte, String), bindings (`let`/`var`).
- Control flow: `if`/`elif`/`else`/`end`, `loop`, `while`, `repeat`, `each` in collections, `break`, `continue`.
- Functions and closures: first-class functions, default parameters, named arguments, lexical closures.
- Data structures: `struct` with defaults and `fixed` fields, `impl` blocks, `List` and `Dict` collections.
- Error handling: recoverable `fail` / `or_else` / `attempt ... failed ... end` (Model B) and runtime faults.
- Contracts: optional signature contracts on parameters and returns checked at boundaries.
- Tooling: `aipo run`, `aipo check`, `aipo fmt`, `aipo build` and structured JSONL diagnostics.
- JavaScript backend (Wave 2, slice W2-1 delivered): Core IR → ESM bundle
  (`app.js` + versioned runtime shim + source map) with VM↔JS differential
  parity over the conformance corpus (see `docs/evidence/P01-G01-js-parity-mvp-subset.md`).

## Non-goals and what is explicitly out of scope

What remains out of scope after Wave 2 slice W2-1:
- Async/await and concurrency primitives (deferred to Wave 3).
- Host embedding ABI, Poppy game engine adapter, and sandboxing (deferred to Wave 4).
- Language Server Protocol (LSP) and REPL (deferred to Wave 5).
- Package manager, registry, and hot reload (deferred to Wave 6).
- `Bytes` packing APIs, `Set`, lazy `Sequence`, regex/json/fs/http modules.
- Experimental optimizations: JIT, Cranelift, NaN-boxing, custom allocators.

## Compatibility constraints

- Implementation edition: Rust Edition 2024.
- Target platforms: Tier 1 Linux (x86_64, aarch64), macOS (Apple Silicon, Intel), Windows (x86_64).
- Safe Rust first: `unsafe_code = "forbid"` across workspace.
- No Rust panic may ever leak as an Aipo runtime error.
