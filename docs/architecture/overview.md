# Aipo — Architecture Overview (Wave 1)

**Status:** normative (architecture level)
**Authority:** subordinate to canonical docs in `docs/canon/`
**Scope:** Wave 1 implementation architecture: pipeline, data ownership, boundaries
**Blocking questions:**
- What owns canonical state?
- Which components may mutate it?

## System boundaries and major components

The Aipo system is partitioned into strictly bounded, acyclic workspace components:

1. **Source & Diagnostics**: `aipo-source` (manages files, byte spans, line indexing) and `aipo-diagnostics` (catalog of stable error codes and JSONL serialization).
2. **Compiler Frontend**: `aipo-lexer` (tokenization), `aipo-syntax` (lossless CST and recovery parser), `aipo-ast` (typed AST), `aipo-hir` (desugaring and lowering), and `aipo-sema` (scope resolution, contracts, mutability).
3. **Target-Neutral Middle-end**: `aipo-ir` (Core IR).
4. **Bytecode & Execution Runtime**: `aipo-bytecode` (instruction encoding and verifier) and `aipo-vm` (stack-based virtual machine).
5. **Tooling & Orchestration**: `aipo-formatter` and `aipo-cli` (`aipo run`, `aipo check`, `aipo fmt`).

## Dependency direction

The dependency direction is strictly unidirectional and acyclic:
`CLI → Runtime / VM → Bytecode → Core IR → Sema → HIR → AST → Syntax → Lexer → Source → Diagnostics`.
No backend or execution layer may be imported into the frontend or syntax tree.

## External integrations

Wave 1 has minimal external integrations:
- Operating system file system (via standard library file reads for `.aipo` source files).
- Standard streams (`stdin`, `stdout`, `stderr`) for program execution and diagnostics.

## Canonical state ownership and mutation

### What owns canonical state?
- **Compile time**: The `SourceMap` in `aipo-source` owns the canonical source text. Each compilation phase produces immutable data artifacts (`Source` → `Tokens` → `SyntaxTree` → `Ast` → `Hir` → `CoreIr` → `BytecodeModule`).
- **Runtime**: The virtual machine `aipo-vm` owns the execution state (call stack, frames, and GC-managed object heap).

### Which components may mutate it?
- Compile-time data structures are append-only or immutable transformations across phase boundaries.
- At runtime, only the VM's active evaluation frame and execution loop may mutate local variables and mutable data structures (`var` bindings, mutable list/dict elements) in accordance with Aipo mutability semantics.

## Pipeline (Wave 1)

```
.aipo file
  → aipo-source     (load, UTF-8, BOM/CRLF normalize, SourceMap)
  → aipo-lexer      (TokenKind + SourceSpan; keywords; f/r/fr strings)
  → aipo-syntax     (lossless tree + recovery)
  → aipo-ast        (typed AST, module structural split)
  → aipo-hir        (early lowering: trailing blocks → fn literals, |>, ellipsis)
  → aipo-sema       (scopes, mutability paths, contracts, modules, interfaces)
  → aipo-ir         (target-neutral Core IR)
  → aipo-bytecode   (instruction selection + encoding + verifier)
  → aipo-vm         (stack interpreter over managed heap)
  → aipo-cli        (aipo run / aipo check / aipo fmt)
```

Diagnostics flow from every stage into `aipo-diagnostics`; user output is either
human-rendered or `--message-format=jsonl` (stable codes, machine-readable).

## Failure vs fault (runtime model, Wave 1)

- **Failure** = recoverable, created by `fail(...)` or runtime operations documented as
  fallible; propagates automatically (Model B); captured by `or_else` / `attempt...failed`.
- **Fault** = programming error (overflow, div by zero, index/key errors, mutation during
  iteration, contract violation discovered at runtime, non-Bool condition); not capturable;
  ends execution with a structured runtime-fault diagnostic. Never a Rust panic.
