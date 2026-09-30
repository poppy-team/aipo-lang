# Evidence — P02-G03 / Wave 3 Infra: Sintaxe `async fn`, `await do` e Diagnósticos Estáticos

**Goal:** `P02-G03` — Implementar e certificar sintaxe `async fn`, desugaring de `await do`, diagnósticos estáticos (`AIPO_SEM_AWAIT_IN_SUBEXPRESSION`, `AIPO_SEM_FORGOTTEN_TASK`, `AIPO_SEM_NESTED_AWAIT_DO`), com testes e conformance.
**Phase:** P02 (Wave 3 — Async & Expanded Types) · **Recorded:** 2026-09-21
**Environment:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (all executed, all green)

```
$ cargo fmt --all -- --check                                           # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings                 # 0 warnings
$ cargo test --workspace --all-targets                                  # 301 passed, 0 failed
$ cargo doc --workspace --no-deps                                       # 0 warnings
$ node crates/aipo-js/runtime/selftest.mjs                              # all assertions passed
$ cargo test -p aipo-cli --test conformance                             # 13 passed (27 programs, 29 diagnostics)
$ cargo test -p aipo-js --test differential                             # 8 passed (VM == JS == stdout)
$ cargo test -p aipo-js --test properties                               # 3 passed
```

`prumo validate` / `prumo doctor` cannot run in this environment: the local prumo 0.6.0
installation is missing its framework resources (`framework-check` reports
`open src/prumo/resources/catalog/catalog.json: no such file or directory` and
`missing schema: prumo.schema.json`). This is an installation defect, independent of the
repository — no project file under version control references those paths.

## Summary of Implementation

| Area | Decisions & Implementation Details |
|---|---|
| **`async fn` as call protocol** | Calling an `async fn` spawns the task eagerly and yields its `Task` handle; `await` is explicit everywhere (canon: no implicit parallelism). Parsing is shared by top-level `Item::Fn`, local `Stmt::Fn`, anonymous `async fn(...)` closures and `impl` methods. |
| **Top-level `async fn` is an item** | `is_item_start` was missing `TokenKind::Async`, so a top-level `async fn` was lowered as a local statement and `AIPO_SEM_FORGOTTEN_TASK` never saw it. Fixed in `crates/aipo-syntax/src/parser.rs`. |
| **Anonymous `async fn`** | `parse_async_function_decl` is used for the value form too; previously the anonymous form never advanced past `fn`. Result: an anonymous `async fn` is a closure with the same call protocol. |
| **`async fn` methods on `impl`** | `register_struct_method` carries an `is_async` flag through the four registration call sites (`aipo-cli`, `aipo-testkit`, `aipo-js` differential, `aipo-vm` data/errors); the VM and the JS runtime spawn the method body with the receiver as argument 0. |
| **`await do … end`** | Desugared as sequential awaits over the block body; nested block form reports `AIPO_SEM_NESTED_AWAIT_DO`, and `await` in arbitrary subexpressions reports `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`. |
| **`AIPO_SEM_FORGOTTEN_TASK`** | A known-`Task` expression used as a bare statement without `await`, binding or combinator is reported; binding and returning a task stay silent. |
| **Static await contract (ADP-006 §G)** | A `Task` is only produced by an `async fn` call or a combinator, so a literal `await` operand is provable before execution and reports `AIPO_SEM_CONTRACT_VIOLATION_STATIC` instead of reaching the runtime fault. |
| **Parametric-contract recovery** | The skip path for `Task[T]`/`List[Int]` consumed the caller's closing `)`, cascading a spurious `AIPO_PARSE_UNEXPECTED_TOKEN` after the real `AIPO_SEM_PARAMETRIC_CONTRACT`; recovery now stops when the section's own bracket closes. |
| **Await cycle and cancellation** | Driving/awaiting a task whose outcome was already consumed re-executed the body forever; the outcome is now absorbed at the boundary and a cycle faults with `AIPO_RT_AWAIT_CYCLE`. Awaiting a cancelled task faults with `AIPO_RT_CANCELLED`. Both implemented in `aipo-vm/src/vm/{failure,mod}.rs` and mirrored in `crates/aipo-js/runtime/aipo-runtime.js`. |
| **Numeric-literal hardening** | A single shared module `aipo-lexer::number` owns the literal rules (bases, separators, float forms, `i64` bounds). The lexer and the IR builder both call it, removing the previous divergence between lexical validation and constant materialization. Missing digits after a base prefix, malformed separators and out-of-range integers report `AIPO_LEX_INVALID_NUMBER`; a literal above the safe-integer range faults with `AIPO_RT_OVERFLOW`. |

## Test Inventory & Verification

Unit and property tests:

- `crates/aipo-lexer/src/lib.rs` and `crates/aipo-lexer/tests/properties.rs`: numeric-literal tables (decimal/hex/binary/octal, separators, boundaries) and a property that any literal the lexer accepts round-trips through the shared classifier.
- `crates/aipo-ir/src/lib.rs`: IR builder builds the same value the lexer classified for every accepted literal form.
- `crates/aipo-sema/src/lib.rs`: the three static diagnostics and the static await contract — each positive case and the silent negative cases (bound task, task returned to the caller, awaited `async fn` call).
- `crates/aipo-vm/tests/wave3_async_semantics.rs`: `async fn` spawn/await, `await do` ordering, forgotten-task/reporting boundaries, await-cycle and cancelled-await faults.
- `crates/aipo-js/tests/properties.rs`: numeric-literal and async emission properties over the corpus.

Conformance corpus (29 diagnostic fixtures total):

- `docs/conformance/programs/24_async_functions_and_await.aipo` — `async fn` top-level, local, anonymous and `impl` method; task bound, passed and joined; `await` as statement, initializer and return value.
- `docs/conformance/programs/25_await_do.aipo` — `await do` sequential sugar over several tasks, including a local `fn` that awaits inside the block.
- `docs/conformance/programs/26_async_combinators.aipo` — `task.all`/`race`/`timeout`/`cancel`/`sleep`/`group` composed with deterministic ordering.
- `docs/conformance/programs/27_numeric_literal_bases.aipo` — every accepted literal base, separator and `i64`-range boundary.
- `docs/conformance/diagnostics/20_lex_invalid_number.aipo` → `AIPO_LEX_INVALID_NUMBER`
- `docs/conformance/diagnostics/21_runtime_overflow_literal.aipo` → `AIPO_RT_OVERFLOW`
- `docs/conformance/diagnostics/22_runtime_overflow_unrepresentable.aipo` → `AIPO_RT_OVERFLOW`
- `docs/conformance/diagnostics/23_sem_await_in_subexpression.aipo` → `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`
- `docs/conformance/diagnostics/24_sem_forgotten_task.aipo` → `AIPO_SEM_FORGOTTEN_TASK`
- `docs/conformance/diagnostics/25_sem_nested_await_do.aipo` → `AIPO_SEM_NESTED_AWAIT_DO`
- `docs/conformance/diagnostics/26_runtime_uncaught_failure_through_await.aipo` → `AIPO_RT_FAILURE_UNCAUGHT`
- `docs/conformance/diagnostics/27_runtime_await_cycle.aipo` → `AIPO_RT_AWAIT_CYCLE`
- `docs/conformance/diagnostics/28_runtime_await_cancelled.aipo` → `AIPO_RT_CANCELLED`
- `docs/conformance/diagnostics/29_sem_await_literal.aipo` → `AIPO_SEM_CONTRACT_VIOLATION_STATIC`

Both suites enumerate the corpus from disk, so every new fixture is exercised by
`cargo test -p aipo-cli --test conformance` and, for differential parity,
`test_vm_and_js_agree_with_committed_stdout` in `crates/aipo-js/tests/differential.rs`.

## Documentation Impact

- `docs/diagnostics/catalog.md`: four new semantic codes and three new runtime fault codes registered.
- `docs/conformance/README.md`: program rows 24–27, diagnostic rows 20–28, the Wave 2/Wave 3
  delivery table and the post-`P02-G03` inventory.
- `CHANGELOG.md`: the `P02-G03` addition and the four correctness fixes.
- `PROJECT_STATE.md`: completed slice and active goal.
- `.ai/goals/P02-G03.goal.json`: objective acceptance criteria, non-goals and state history.

## Residuals

- `AIPO_RT_AWAIT_IN_CALLBACK` is exercised by the Wave 3 stdlib evidence (`P02-G02`); this goal
  does not add a fixture for it.
- Wave 3 exit criteria in `docs/waves/wave-3-async.md` are met for syntax, diagnostics,
  cancellation faults and VM↔JS parity; Wave 4 (host ABI/Poppy) remains out of scope.
