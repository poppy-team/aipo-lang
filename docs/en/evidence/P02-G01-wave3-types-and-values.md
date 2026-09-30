# Evidence — P02-G01 / Wave 3 Tipos e Valores (Set, Sequence, Bytes Packing, Task, Duration)

**Goal:** `P02-G01` — Implementar os tipos/valores da Wave 3 (Set com ordem de inserção, Sequence lazy, packing de Bytes little-endian, Task como valor, Duration) com tags, prelude, display/igualdade e testes, sem mudar semântica existente.
**Phase:** P02 (Wave 3 — Async & Expanded Types) · **Recorded:** 2026-09-20
**Environment:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (all executed, all green)

```
$ cargo fmt --all -- --check                                           # 0
$ cargo clippy --workspace --all-targets -- -D warnings                 # 0
$ cargo test --workspace                                               # 100% passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 (0 warnings)
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-js --test differential                            # 3 passed (VM == JS == stdout)
$ prumo validate .                                                     # clean
$ prumo doctor .                                                       # clean
```

## Summary of Implementation

| Area | Decisions & Implementation Details |
|---|---|
| **Set** | `Value::Set(Rc<RefCell<Vec<Value>>>)` with insertion order preserved, set-theoretic equality (`==`), formatting `Set([a, b, ...])`. Methods: `has(val)`, `add(val)`, `remove(val)`, `clear()`, `is_empty()`, `len()`, `to_list()`, `lazy()`. Conversion call `Set(list)` deduplicates in first-occurrence order. Exact mirror in `aipo-runtime.js` (`vSet`, methods). |
| **Sequence** | `Value::Sequence(SequencePipeline)` represents an immutable lazy iterator chain starting from a List, Dict (values), or Set. Lazy pipeline operations: `map`, `filter`, `flat_map`, `take`, `skip`, `distinct`, `chain`, `enumerate`. Terminal consumers: `collect` (returns `List`), `find`, `any`, `all`, `count`, `reduce`, `group_by`. Indexing drains and evaluates elements. Mirrored in `aipo-runtime.js`. |
| **Bytes Packing** | Refactored `Value::Bytes` to `Rc<RefCell<Vec<u8>>>` to support in-place mutation and exact index writes. Implemented receiver-first binary packing methods with little-endian ordering: `read_i8`, `read_u8`, `read_i16`, `read_u16`, `read_i32`, `read_u32`, `read_i64`, `read_u64`, `read_f32`, `read_f64`, and corresponding `write_*` methods. UTF-8 conversion: `String.encode()` -> `Bytes`, `Bytes.decode()` -> `String` (with recoverable `Failure` on invalid UTF-8). Out-of-bounds access faults deterministically. Parity in JS backend using `DataView` with little-endian flag (`true`). |
| **Duration** | `Value::Duration(f64)` representing seconds with microsecond-level precision. Supports arithmetic (`+`, `-`, unary `-`), relational comparison (`<`, `<=`), equality, and method `.total_seconds()`. Conversion `Duration(seconds)` accepts `Int` or `Float`. Exact mirror in JS runtime (`vDuration`). |
| **Task & Group** | `Value::Task(TaskId)` and `Value::Group(GroupId)` representing async task handles. Registered as built-in contract types in `aipo-sema` and `TypeTag` in `aipo-vm` without polluting the global value namespace as constructors (preventing collisions with user structs named `Task`, as verified by conformance `programs/10_integrated.aipo`). Module `task` provides built-in combinators: `spawn`, `sleep`, `all`, `race`, `timeout`, `cancel`, `group`. |
| **JS Backend Parity** | `crates/aipo-js/runtime/aipo-runtime.js` mirrors 100% of the types, methods, tag matches, type conversions, and module dictionaries. Validated via `crates/aipo-js/runtime/selftest.mjs` and `crates/aipo-js/tests/differential.rs`. |

## Test Inventory & Verification

- `crates/aipo-stdlib/tests/wave3_types.rs`: Unit tests covering Set mutation and deduplication, Sequence lazy chains, little-endian binary packing across all numeric widths (`i8`..`f64`), String UTF-8 encode/decode, and Duration arithmetic.
- `crates/aipo-vm/tests/wave3_pipeline.rs`: End-to-end VM integration pipeline tests compiling Aipo source with Wave 3 constructs directly to IR, bytecode, and execution.
- `crates/aipo-js/tests/differential.rs`: Differential testing verifying that programs executing Wave 3 types and operations produce identical standard output and exit codes across the VM and Node.js runners.
