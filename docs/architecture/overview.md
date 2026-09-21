# Aipo — Architecture Overview (Waves 1–2)

**Status:** normative (architecture level)
**Authority:** subordinate to canonical docs in `docs/canon/`
**Scope:** Wave 1–2 implementation architecture: pipeline, data ownership, boundaries
**Blocking questions:**
- What owns canonical state?
- Which components may mutate it?

## System boundaries and major components

The Aipo system is partitioned into strictly bounded, acyclic workspace components:

1. **Source & Diagnostics**: `aipo-source` (manages files, byte spans, line indexing) and `aipo-diagnostics` (catalog of stable error codes and JSONL serialization).
2. **Compiler Frontend**: `aipo-lexer` (tokenization), `aipo-syntax` (lossless CST and recovery parser), `aipo-ast` (typed AST), `aipo-hir` (desugaring and lowering), and `aipo-sema` (scope resolution, contracts, mutability).
3. **Target-Neutral Middle-end**: `aipo-ir` (Core IR).
4. **Bytecode & Execution Runtime**: `aipo-bytecode` (instruction encoding and verifier) and `aipo-vm` (stack-based virtual machine over `Rc<RefCell<…>>` shared values — single-threaded by decision, no GC crate).
5. **Runtime services & Stdlib**: `aipo-runtime` (module registry, native registry) and `aipo-stdlib` (Prelude, `math`, `string`, `io`, collection methods).
6. **JavaScript backend (Wave 2)**: `aipo-js` (Core IR → ESM bundle + versioned runtime shim + source maps; depends on `aipo-ir` only, never on bytecode).
7. **Tooling & Orchestration**: `aipo-formatter` and `aipo-cli` (`aipo run`, `aipo check`, `aipo build`, `aipo fmt`).
8. **Test-only harnesses**: `aipo-testkit` (deterministic RNG, program generator, pipeline/JS runners — never ships language semantics) and `aipo-bench` (benchmark runner binary).

## Dependency direction

The dependency direction is strictly unidirectional and acyclic:
`CLI → Runtime / VM → Bytecode → Core IR → Sema → HIR → AST → Syntax → Lexer → Source → Diagnostics`,
with `aipo-js` branching off Core IR (`JS → Core IR` only).
No backend or execution layer may be imported into the frontend or syntax tree.

## External integrations

Wave 1 has minimal external integrations:
- Operating system file system (via standard library file reads for `.aipo` source files).
- Standard streams (`stdin`, `stdout`, `stderr`) for program execution and diagnostics.

## Canonical state ownership and mutation

### What owns canonical state?
- **Compile time**: The `SourceMap` in `aipo-source` owns the canonical source text. Each compilation phase produces immutable data artifacts (`Source` → `Tokens` → `SyntaxTree` → `Ast` → `Hir` → `CoreIr` → `BytecodeModule`, or `CoreIr` → JS bundle).
- **Runtime**: The virtual machine `aipo-vm` owns the execution state (call stack, frames, and `Rc<RefCell<…>>` shared values); the JS backend mirrors the same value model in its versioned shim.

### Which components may mutate it?
- Compile-time data structures are append-only or immutable transformations across phase boundaries.
- At runtime, only the VM's active evaluation frame and execution loop may mutate local variables and mutable data structures (`var` bindings, mutable list/dict elements) in accordance with Aipo mutability semantics.

## Pipeline (Waves 1–2)

```
.aipo file
  → aipo-source     (load, UTF-8, BOM/CRLF normalize, SourceMap)
  → aipo-lexer      (TokenKind + SourceSpan; keywords; f/r/fr strings)
  → aipo-syntax     (lossless tree + recovery; recursion bounds per ADP-005)
  → aipo-ast        (typed AST, module structural split)
  → aipo-hir        (early lowering: trailing blocks → fn literals, |>, ellipsis)
  → aipo-sema       (scopes, mutability paths, contracts, modules, interfaces)
  → aipo-ir         (target-neutral Core IR)
  → ┬→ aipo-bytecode (instruction selection + encoding + verifier)
  │   → aipo-vm     (stack interpreter; aipo-runtime + aipo-stdlib linked in)
  └──→ aipo-js       (ESM bundle + versioned shim + source maps)
  → aipo-cli        (aipo run / aipo check / aipo build / aipo fmt)
```

Diagnostics flow from every stage into `aipo-diagnostics`; user output is either
human-rendered or `--message-format=jsonl` (stable codes, machine-readable).

## Failure vs fault (runtime model)

- **Failure** = recoverable, created by `fail(...)` or runtime operations documented as
  fallible; propagates automatically (Model B); captured by `or_else` / `attempt...failed`.
- **Fault** = programming error (overflow, div by zero, index/key errors, mutation during
  iteration, contract violation discovered at runtime, non-Bool condition); not capturable;
  ends execution with a structured runtime-fault diagnostic. Never a Rust panic.
