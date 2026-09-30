# Evidence — P00-G10 / Backend Completion (gaps inside S1–S9)

**Goal:** `P00-G10` — Backend Completion: functions, closures, collections, Byte/Range/slice, and module execution
**Phase:** P00 (Foundation) · **Recorded:** 2026-09-15
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.

## Why this goal existed

Slices S1–S9 delivered each stage of the pipeline with its own tests, but the stages were not yet
wired to each other end to end: the parser produced calls that the bytecode emitter could not
lower, the VM had no first-class function values, the collections API stopped at the operations
that need no callback, and `import`/`export` were specified in S9 but never executed. This goal
closes those gaps so that a `.aipo` program can use the full MVP subset through `aipo run`.

## What changed, by stage

| Stage | Crate | Gap closed |
|---|---|---|
| Frontend | `aipo-syntax` | `f"..."` interpolation desugars to `String(...)` concatenation; `a[a..b]`, `a[..b]`, `a[a..]` and `a[..]` parse into range-index expressions instead of a bare `a[a]` |
| Core IR | `aipo-ir` | struct declarations carried on `Program`; `BuildStruct` initializes fields in canonical order; two new opcodes for default parameters and the callee-side prologue |
| Bytecode | `aipo-bytecode`, `aipo-bytecode::module` | struct table on `BytecodeModule`; emitter, verifier and disassembler cover every new opcode |
| VM | `aipo-vm` | first-class function values and closures with upvalues; `Value::Byte`/`Value::Bytes`/`Value::Type`; type tests; range and slice indexing; receiver-aware call frames; `MutationDuringIteration` enforcement on `List`/`Dict` iteration |
| Stdlib | `aipo-stdlib` | `collections.rs` binds `List`/`Dict` methods to the VM; conversions delegate to the VM so `String(v)` and `io.print(v)` cannot diverge; `String`/`Bytes` method surface |
| Sema | `aipo-sema` | Prelude surface (`docs/…/mvp-subset.md`) is visible to name resolution |
| CLI | `aipo-cli` | `modules.rs` resolves and executes `import`/`export` with a topological order and init-once semantics |

## Gate: `fmt`

```
$ cargo fmt --all -- --check
FMT_EXIT=0
```

## Gate: `check`

```
$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Gate: `clippy` (workspace lints deny warnings)

```
$ cargo clippy --workspace --all-targets -- -D warnings
(no output, exit 0)
```

## Gate: `test`

```
$ cargo test --workspace
passed: 136  failed: 0
```

Per crate:

| Crate | Tests | | Crate | Tests |
|---|---|---|---|---|
| `aipo-source` | 5 | | `aipo-ir` | 2 |
| `aipo-diagnostics` | 3 | | `aipo-bytecode` | 4 |
| `aipo-lexer` | 5 | | `aipo-vm` | 24 |
| `aipo-syntax` | 13 | | `aipo-runtime` | 10 |
| `aipo-ast` | 1 | | `aipo-stdlib` | 30 |
| `aipo-hir` | 4 | | `aipo-formatter` | 11 |
| `aipo-sema` | 8 | | `aipo-cli` | 16 |

The gap-closing work is pinned by the conformance corpus rather than by unit tests alone. Each
program fixture in `docs/conformance/programs/` is executed through the real pipeline and compared
against committed stdout, and each diagnostic fixture must fail with its committed code:

| Fixture | Gap it proves closed |
|---|---|
| `02_recursion`, `11_defaults_and_named_args` | functions with real locals/params, default parameters evaluated per call, defaults referencing earlier parameters, named arguments in any order, pipeline into a call |
| `06_closures` | anonymous `fn`, capture, per-iteration capture in loops |
| `04_collections` | `List`/`Dict` method API including the higher-order `transform`, `filter` and `sort_by` (which call back into Aipo through the VM) |
| `09_slicing` | ranges `a..b`, list/string slicing with negative bounds, omitted slice bounds |
| `05_structs_and_impl` | `struct` with defaults, construction, `impl` methods, associated functions |
| `07_strings_and_math` | `Byte(255)`, `string`/`math` surface, `f"..."` interpolation |
| `diagnostics/08_runtime_mutation_during_iteration` | `AIPO_RT_MUTATION_DURING_ITERATION` |
| `modules/basic`, `modules/cycle`, `modules/missing` | import/export execution, init-once, privacy, `AIPO_SEM_IMPORT_CYCLE`, `AIPO_SEM_UNKNOWN_MODULE` |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
   Generated /home/raillen/Documentos/Projetos/aipo-lang/target/doc/aipo_ast/index.html and 14 other files
```

## Gate: `documentation_impact`

- `docs/stdlib/mvp-subset.md` — module execution is no longer listed as S9-only; the `List`/`Dict`
  method surface and the reason the higher-order methods live in the VM are recorded.
- `docs/adp/ADP-001-byte-and-core-types-as-values.md` — Q1 and Q2 are resolved by this slice
  (`Value::Byte` and first-class type values now exist); the remaining open questions stay open.
- `docs/evidence/P00-G12-conformance-and-mvp-gate.md` — the corpus this slice validates is scored
  there.
- `CHANGELOG.md`, `PROJECT_STATE.md`.
