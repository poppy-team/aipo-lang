# Evidence — P00-G16 / Wave 1 Exit Review

**Goal:** `P00-G16` — audit the certified MVP subset against the exit criteria and hand it over with a gauntlet score
**Phase:** P00 (Foundation) · **Recorded:** 2026-09-19
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.
The corpus specification lives in `docs/conformance/README.md`; this record is the score and the hand-off, not a second specification.

## Why this goal existed

`P00-G15` closed the last three residuals (`AIPO_SEM_CONTRACT_VIOLATION_STATIC`, structural interface conformance, ADP-001 Q3/Q4/Q5) and both ADP documents became `resolved`. The remaining risk was not a language gap but a hand-off gap: no single record classified every Wave 1 exit criterion in `docs/waves/wave-1-mvp.md` as certified vs. explicitly not delivered, with the command that proves each claim.

`P00-G16` also carried two semantic completions found during the audit window (module-scope visibility and canonical local `fn`), certified by `programs/19_local_functions` and `programs/20_module_scope`. This record covers both the code baseline and the audit.

## Certified baseline

| Kind | Count | Location |
|---|---|---|
| Runnable programs + stdout snapshots | 20 + 20 | `docs/conformance/programs/` (`01`–`20`) |
| Diagnostic fixtures + expected codes | 19 + 19 | `docs/conformance/diagnostics/` (`01`–`19`) |
| Formatter golden pairs | 8 + 8 | `docs/conformance/formatting/` |
| Module cases | 3 | `docs/conformance/modules/` (`basic`, `cycle`, `missing`) |

Inventory verified on disk (`20/20`, `19/19`, `8/8`, `3` entry points). No orphan `.stdout` without `.aipo`, no `.code` without `.aipo`.

New certification since `P00-G15`:

| Artifact | Proves |
|---|---|
| `docs/conformance/programs/19_local_functions.aipo` | local `fn` declarations: shared `var` capture (`create_counter`), self-recursion via `FillSelfCapture`, lexical capture of enclosing parameter |
| `docs/conformance/programs/20_module_scope.aipo` | module-scope bindings visible inside `fn` bodies: `var total` mutated by `add`, `let scale` read by `scaled` |

Direct run:

```
$ cargo run -q -p aipo-cli -- run docs/conformance/programs/19_local_functions.aipo
3
120
small
large

$ cargo run -q -p aipo-cli -- run docs/conformance/programs/20_module_scope.aipo
total: 5
40
```

Both match their committed `.stdout`.

## Gauntlet rubric — 100%

| Gate | Weight | Command | Result |
|---|---|---|---|
| Executable MVP | 30% | `cargo test -p aipo-cli --test conformance` | **pass** — 13/13, all program and module fixtures match committed stdout |
| Diagnostic discipline | 20% | same suite, failure tests | **pass** — every fixture fails with its committed code; wrong-reason failures would fail the suite |
| Determinism | 20% | `cargo test -p aipo-formatter` + `fmt --check` corpus tests | **pass** — 6 + 4 + 1 = 11/11; formatter idempotent, canonical sources report no drift |
| Robustness | 15% | `cargo test -p aipo-cli --test fuzz_smoke` | **pass** — 3/3; random bytes, mutated and truncated programs never panic |
| Repository gates | 15% | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps` | **pass** — all four green |

Command output:

```
$ cargo test -p aipo-cli --test conformance
test result: ok. 13 passed; 0 failed

$ cargo test -p aipo-formatter
test result: ok. 6 passed; 0 failed
test result: ok. 4 passed; 0 failed
test result: ok. 1 passed; 0 failed

$ cargo test -p aipo-cli --test fuzz_smoke
test result: ok. 3 passed; 0 failed

$ cargo test --workspace
PASSED=146 FAILED=0
```

## Gate: `fmt` / `clippy` / `test` / `doc` / `prumo`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_EXIT=0 (warnings: 0)

$ cargo test --workspace
PASSED=146 FAILED=0

$ cargo doc --workspace --no-deps
DOC_EXIT=0 (warnings: 0)

$ prumo validate .
Prumo validation passed.

$ prumo doctor .
Prumo Doctor: all checks passed cleanly.
```

Additional surface checks:

```
$ cargo run -q -p aipo-cli -- check docs/conformance/programs/20_module_scope.aipo
CHECK_EXIT=0

$ cargo run -q -p aipo-cli -- check docs/conformance/diagnostics/15_sem_contract_violation.aipo
error: [AIPO_SEM_CONTRACT_VIOLATION_STATIC] argument for parameter 'x' of 'double' cannot satisfy contract 'Int'
CHECK_FAIL_EXIT=1

$ cargo run -q -p aipo-cli -- --version
aipo 0.1.0
```

## MVP exit gate — every criterion classified

`docs/waves/wave-1-mvp.md` lists five exit conditions. Status against each:

| Exit condition | Status | Evidence |
|---|---|---|
| `aipo run` executes real small programs | **certified** | `programs/01`–`20` snapshots; `cargo test -p aipo-cli --test conformance` 13/13 |
| `aipo check` reports diagnostics; `aipo fmt` idempotent | **certified** | `diagnostics/01`–`19` codes; `formatting/01`–`08` goldens + `cargo test -p aipo-formatter` 11/11 |
| pass/fail fixtures green; snapshots committed; integration tests green | **certified** | conformance 13/13; `cargo test --workspace` 146/146 |
| No Rust panic escapes as user error (fuzz smoke) | **certified** | `cargo test -p aipo-cli --test fuzz_smoke` 3/3 |
| CI: fmt/clippy/test/doc green; `prumo validate` + `prumo doctor` green; docs delta resolved | **certified** | gates above; `prumo` outputs above; delta table below |

MVP language subset (`docs/waves/wave-1-mvp.md`, formal recorte) — all items certified by the corpus or the unit suites:

| Subset area | Status | Representative proof |
|---|---|---|
| Literals (`none`/`true`/`false`, Int ±(2^53−1), Float finite, `Byte`, `"…"`, `f/r/fr`, `"""`) | **certified** | `programs/01_hello`, `07_strings_and_math`, `17_unicode_nfc` |
| Bindings (`let`/`var`, `var`/`!`/`self!`/`fixed` paths, destructuring surface) | **certified** | `programs/05_structs_and_impl`, `10_integrated`, `19_local_functions`, `20_module_scope` |
| Operators (`.`, `?.`, call/index, `* / div %`, `+ -`, `..`, comparisons, `is`, `not`/`and`/`or`, `or_else`, `\|>`, compound assign) | **certified** | `programs/03_control_flow`, `04_collections`, `07_strings_and_math`, `09_slicing` |
| Control flow (`if/elif/else`, inline `if/then`, `match/when`, `loop`/`while`/`repeat`/`each`, `break`/`continue`) | **certified** | `programs/03_control_flow`, `10_integrated` |
| Functions (`fn`, defaults, named args, local `fn`, anonymous `fn`, closures + per-iteration capture, `return`) | **certified** | `programs/02_recursion`, `06_closures`, `11_defaults_and_named_args`, `19_local_functions` |
| Calls (positional-before-named, trailing `do…end`, pipeline, dot-call for `impl` assoc fns) | **certified** | `programs/06_closures`, `10_integrated`, `11_defaults_and_named_args` |
| Data (`struct`+defaults+`fixed`, `Type{…}`, `impl`+`init`+`invariant`, `List`/`Dict` ops, negative indices, `a..b`) | **certified** | `programs/04_collections`, `05_structs_and_impl`, `09_slicing`, `12_bytes`, `13_init_and_invariant`, `15_invariant_on_mutation` |
| Errors (`fail`, `or_else`, `attempt/failed`, `err.message`; numeric/index/key/iteration/contract faults) | **certified** | `programs/08_failures`, `14_signature_contracts`, `15_invariant_on_mutation`, `diagnostics/06`–`10`, `14` |
| Contracts (param/return `Type`, `!`, `-> T`, `T?`; runtime boundary checks; `is` narrowing; interfaces + `satisfy` structural) | **certified** | `programs/14_signature_contracts`, `16_interface_contracts`, `diagnostics/12/13/15/16/17/18` |
| Modules (one file = one module; `import m`, `import m: names`, `import m as alias`, `export`, acyclic, init-once) | **certified** | `modules/basic`, `modules/cycle`, `modules/missing` |
| Prelude + stdlib (`len/copy/same/some/fail`, conversions, `io`, `string`, `List`/`Dict`, `math`) | **certified** | `programs/01_hello`, `04_collections`, `07_strings_and_math`, `17_unicode_nfc`, `18_tolerant_slices_and_clamp` |
| CLI (`run`/`check`/`fmt`, `--message-format=jsonl`, exit codes `0/1/2`) | **certified** | conformance suite (`--help`/`--version`, jsonl, usage-code tests) |

## Explicit non-delivery (not gaps, not regressions)

What the MVP subset declares out of scope stays out of scope. No later slice inherits a hidden gap:

| Non-delivery | Scope source | Status |
|---|---|---|
| `Bytes` packing APIs (`read_i32`/`write_f32`/…, `String.encode`/`Bytes.decode`) | `docs/waves/wave-1-mvp.md` exclusions; `docs/stdlib/mvp-subset.md` Deferred | **not delivered by design** — only `Bytes(count)` + index/`len`/slice are certified (`programs/12_bytes`) |
| `aipo-js` backend | crate contracts (Wave 2); MVP exclusions | **not delivered** — pipeline is bytecode → VM only; crate not started |
| `Set` / lazy `Sequence` | MVP exclusions; stdlib Deferred | **not delivered** — no kind, no literal, no fixture claims them |
| LSP / REPL | MVP exclusions | **not delivered** — CLI is `run`/`check`/`fmt` only |
| async/await, host ABI/Poppy, packages/registry, regex/json/fs/http, `graphemes`, hot reload | MVP exclusions | **not delivered** — recorded here so Wave 2 planning starts from an explicit list |

Both ADP documents stay `resolved` and are not re-opened by this goal.

## Recommended next slice

Wave 1 is closed. The certified baseline above is the starting point. Recommended ordering for Wave 2 planning (no Wave 2 code started by this goal):

1. `aipo-js` backend spike against the frozen corpus (`programs/01`–`20` as the parity oracle) — smallest slice that proves the second backend without touching the language surface.
2. `Bytes` packing APIs only after the JS parity decision, because encode/decode crosses the NFC boundary documented in ADP-001 Q5.
3. `Set`/`Sequence` and LSP/REPL after the backend question, in that order — each is independent of the others once the corpus is the referee.

## Gate: `documentation_impact`

| Artifact | Change |
|---|---|
| `docs/conformance/README.md` | gauntlet scores with exact commands; inventory verified (`20/20`, `19/19`, `8/8`, 3 module cases); non-delivery list explicit |
| `docs/stdlib/mvp-subset.md` | Deferred surface lists `Bytes` packing, `aipo-js`, `Set`/`Sequence`, LSP/REPL as explicit non-delivery |
| `docs/evidence/P00-G16-wave-1-exit-review.md` | new — this hand-off record |
| `docs/PRUMO.md` | evidence index gains `P00-G16` |
| `CHANGELOG.md`, `PROJECT_STATE.md` | Wave 1 exit recorded; `P00-G16` DONE |

## Known limitations (recorded, not hidden)

- **The audit certifies the subset, not every combination.** The corpus plus 146 unit/integration tests is the referee; untested feature interactions outside the corpus are still subject to the no-invention policy and become ADPs when found.
- **Static contracts stay provable-only.** `AIPO_SEM_CONTRACT_VIOLATION_STATIC` fires for literals against core-type contracts and `none` against non-nullable ones; everything else stays a runtime contract fault at the call boundary.
- **Interface conformance is name + caller-visible arity.** A matching name with an incompatible parameter type is discovered when the operation runs, not at the contract check.
- **`prumo` gates are environment-provided.** `prumo validate` / `prumo doctor` are green on this machine; CI must run the same four cargo gates plus these two.
