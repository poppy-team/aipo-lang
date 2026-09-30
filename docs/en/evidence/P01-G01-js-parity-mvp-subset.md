# Evidence — P01-G01 / JS Parity for the Full MVP Subset

**Goal:** `P01-G01` — ship the `aipo-js` emitter (ESM + versioned shim + ECMA-426
source maps) and `aipo build`, with a VM↔JS differential suite green over the frozen
Wave 1 corpus
**Phase:** P01 (JavaScript parity, Wave 2) · **Recorded:** 2026-09-19
**Environment:** rustc 1.98.1, cargo 1.98.1, Linux, Node v24.18.0 (suite requires Node ≥ 20)

Proof attachments for the required gates. Commands are reproducible from the repository root.
The Wave 2 plan lives in `docs/waves/wave-2-js-parity.md`; this record is the score, not a
second specification. No language surface change in this goal.

## Why this goal existed

Wave 1 closed with a certified bytecode→VM pipeline and an explicit non-delivery list headed
by the `aipo-js` backend. Canon (Fechamento Arquitetural §10) requires the JS backend to
preserve Aipo semantics — `Int` range, `String` NFC, ordered `Dict`, `Failure`/fault split —
never JS accidentals, with divergence between backends treated as a bug. This goal delivers
the smallest slice that proves it: the full MVP subset (`programs/01`–`20`) running
identically on both backends.

## Implementation

| Area | Decision |
|---|---|
| Emitter input | Target-neutral Core IR (`aipo-ir`); `aipo-js` depends on `aipo-ir` + `aipo-diagnostics` (+ `serde_json` for bundle encoding) only — no bytecode coupling, per crate contracts |
| Code generation | The bundle carries the module as data (`app.js` embeds the Core IR JSON and calls `runModule`); semantics live once, in the versioned shim (`RUNTIME_VERSION = 1.0.0`, recorded in `app.js`, `aipo-js::RUNTIME_VERSION`, shim `RUNTIME_VERSION`) |
| Interpreter core | Stack machine over Core IR ops in `crates/aipo-js/runtime/aipo-runtime.js`: frames with `vars`/`cells`, absolute jump targets, handler stack, per-frame mutation journal, iteration-guard stack, 1024 stack limit — mirroring `aipo-vm` algorithms so parity holds by construction |
| Scope resolution (defect found and fixed during the slice) | Closure bodies address captures with plain `Load`/`Store`; the builder records the names in `upvalues` but the emitter resolves by layout. First version read cells only for `GetUpvalue` and broke shared `var` capture (`programs/06`: counter stuck at 0). Fixed: `Load`/`Store` resolve params/locals → upvalue cells → globals, exactly like the bytecode emitter's `Slot` fallback |
| Function globals | The JS run pre-registers every dotted-less function as a global, mirroring the bytecode prologue (`MakeFunction`+`SetGlobal` per function); module-scope bindings stay globals, so nested functions resolve them by name |
| Construction checks | `AssertInvariant` pops the `Bool` and pushes nothing (first version pushed `none`, which `SealStruct` then mistook for the instance — `programs/13` faulted). Matches the VM pop semantics |
| Failure text parity | Every recoverable message matches the Rust source byte-for-byte (`invalid integer text: "…"`, `math.clamp bounds are inverted: …`, `missing format value for placeholder {…}`, …), because `err.message` is observable program output |
| `Bytes` strictness | `Bytes(x)` accepts only `Int` (no `Byte` widening), like `convert_bytes`; out-of-range message matches (`outside the constructible range 0..=67108864`) |
| Source maps | Valid ECMA-426 map per bundle (`version: 3`, `sources: [<entry>.aipo]`, `sourcesContent`, per-line VLQ mappings); `app.js` ends with `sourceMappingURL=app.js.map` |
| Exit codes | `runModule` returns `1` on fault/uncaught failure and `app.js` calls `process.exit(code)` (first version always exited 0) |
| `aipo build` | New stable command `aipo build <path> [--out <dir>] [--message-format=…]` sharing the `run`/`check` frontend (parse → module resolve → sema → IR). Check-time failures exit 1 with identical codes and emit nothing; runtime faults build fine and surface under `node` with identical codes |

## Gate: differential parity

```
$ cargo test -p aipo-js --test differential
test result: ok. 2 passed; 0 failed   # 20/20 programs: VM == JS == committed .stdout

$ cargo test -p aipo-cli --test js_build
test result: ok. 3 passed; 0 failed   # bundle+node snapshots; 19/19 diagnostics
                                      # (check-time rejected at build, runtime faults identical under node);
                                      # modules/basic bundled, cycle/missing rejected
```

## Gate: `fmt` / `clippy` / `test` / `doc` / `prumo`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_EXIT=0 (warnings: 0)

$ cargo test --workspace
PASSED=152 FAILED=0   # was 146; +1 aipo-js emitter, +2 differential, +3 js_build

$ cargo doc --workspace --no-deps
DOC_EXIT=0 (warnings: 0)

$ prumo validate .
Prumo validation passed.

$ prumo doctor .
Prumo Doctor: all checks passed cleanly.
```

## Gate: `documentation_impact`

| Artifact | Change |
|---|---|
| `docs/waves/wave-2-js-parity.md` | new — Wave 2 objective, architecture, frozen parity subset, slices, exit gate |
| `docs/reference/cli.md` | `build` is stable since Wave 2, with bundle layout, parity rule and Node requirement |
| `docs/crates/crate-contracts.md` | `aipo-js` marked delivered by `P01-G01` (deps, bundle layout, differential suites) |
| `docs/evidence/P01-G01-js-parity-mvp-subset.md` | new — this hand-off record |
| `docs/PRUMO.md` | evidence index gains `P01-G01` |
| `CHANGELOG.md`, `PROJECT_STATE.md`, `prumo.json` | Wave 2 executing under `P01-G01` |

## Known limitations (recorded, not hidden)

- **The shim interprets Core IR data; it is not an optimizing code generator.** Per-function
  JS codegen, bundle minification and performance baselines (`aipo bench` JS workload) are
  explicitly future work — parity first, performance later.
- **`f64` Display edge cases in messages.** Integral floats render `x.0` on both backends;
  extreme magnitudes (`1e21`) may format differently across toolchains. No corpus fixture
  covers that range; divergence there would become an ADP, not a silent fix.
- **Static contract diagnostics stay frontend-only.** `AIPO_SEM_CONTRACT_VIOLATION_STATIC`
  is reported by `build` at build time (verified by diagnostics 15/16 in the build suite);
  it is never a JS runtime event.
- **Remaining Wave 2+ non-delivery is unchanged:** `Bytes` packing APIs, `Set`/`Sequence`,
  LSP/REPL, async, host ABI/Poppy, packages — still out of scope, still explicit.
