# Evidence — P01-G02 / Deep Quality Gauntlet

**Goal:** `P01-G02` — reproducible performance, security, conformance,
accessibility and generated-program validation without changing canonical
language semantics
**Phase:** P01 · **Recorded:** 2026-09-20
**Environment:** Linux x86_64 (4 CPU, 5 GiB RAM), rustc/cargo 1.98.1, Node
v24.18.0, git a8709881 (work committed after)

## Gates (all executed, all green)

```
$ cargo fmt --all -- --check                                   # 0
$ cargo check --workspace --all-targets                        # 0 (7.6s)
$ cargo clippy --workspace --all-targets --all-features -- -D warnings  # 0
$ cargo test --workspace --all-targets                         # 253 passed, 0 failed, 46 suites
$ cargo doc --workspace --no-deps                              # 0
$ prumo validate . && prumo doctor .                           # both clean
$ cargo +1.85 check --workspace --all-targets (MSRV)           # 0
$ cargo audit                                                  # 0 advisories / 31 deps
$ cargo deny check                                             # advisories+bans+licenses+sources ok
$ cargo +nightly miri test -p aipo-vm                          # 44 passed
```

## Test inventory delta (165 → 253)

| Area | Added |
|---|---|
| Testkit (`aipo-testkit`, new) | Rng, corpus discovery, pipeline/js runners, portable watchdog, timing, metamorphic transforms, AipoSmith generator + validity tests |
| Benchmarks (`aipo-bench`, new) | 9 frontend stages × 4 sizes, 4 commands, 16 VM workloads, 2 JS workloads, 3 scaling analyses; `docs/performance/baseline.md` |
| Properties | lexer (3), source (6), syntax token totality (2), value model (7), contracts matrix (2), stdlib unicode (10), fmt equivalence (2) |
| Generated differential | `generated.rs` (80 programs VM==JS), `metamorphic.rs` (4) |
| Security | `hostile.rs` (11), `resource.rs` (9), `unicode_security.rs` (8) |
| UI/accessibility | `diagnostic_ui.rs` (4 + 10 goldens), `suggestions.rs` (3) |
| Backend conformance | `sourcemap.rs` (4), `determinism.rs` (2), `properties.rs` (+VLQ coherence), shim selftest extensions |
| Emitter/VM | overflow-guard tests (3), arity regression (1), DictMap (5), depth tests (3+2) |
| Examples | 19 new examples + `examples.rs` harness (23 cases × VM+JS) |
| Fuzz | 3 libFuzzer targets (build green); grammar + portable run fuzz in `fuzz_smoke` |

## Defects found and fixed (all with regression artifacts)

1. **Host abort on deep nesting (SIGABRT, exit 134)** — recursive-descent
   parser overflows the host stack past ~500–800 paren levels / ~300–600 block
   levels. Fixed: expression/block depth guards + progress guarantee +
   `AIPO_PARSE_NESTING_TOO_DEEP` (ADP-005). Proven by `resource.rs` on 2 MiB
   test threads.
2. **Latent hang in body loops** — any non-consuming parse failure spins `while`
   recovery loops forever (found while fixing 1). Fixed by the same progress
   guarantee, proven not to alter terminating inputs by construction + unchanged
   snapshots.
3. **IR builder panic on valid programs** — constructing a struct with `init`
   inside a parameterized function panicked (`hidden receiver local`,
   `builder.rs:415`). Found by libFuzzer in ~2k executions. Fixed via
   `hidden_name`; regression: `programs/23`.
4. **VM↔JS journal divergence on `attempt` recovery** — found by hand analysis
   during gauntlet review, proven with a probe (VM `2` vs JS `1`), fixed in the
   shim; regression: `programs/21`.
5. **Signed-zero divergence** — `-0.0` display and `Int` negative zero
   (`-33 * 0 === -0` in JS, impossible in Rust i64). Found by generated
   differential (seed 1001). Fixed in shim (`floatText`, `normInt`);
   regression: `programs/22` + selftest assertions.
6. **Quadratic f-string parsing** — per-placeholder `" ".repeat(offset)`
   padding measured 68× per 10× input (23.4 ms for 400 f-strings). Fixed by
   unpadded slices + span shifting (`shift.rs`): 1.77 ms, snapshots identical.
7. **Rotted examples** — `03` (invalid `return` in `invariant`), `04`
   (leading-`|` pipelines, nonexistent `fold`) failed on current main. Fixed;
   examples harness prevents recurrence.
8. **Unix-only `timeout` in fuzz harness** — replaced with portable
   `testkit::proc::run_bounded` (poll + `Child::kill`).

## ADPs opened (ambiguities documented, semantics unchanged)

- **ADP-003 (draft)** — execution budgets: only the stack-depth limit exists;
  fuel/memory/interruption undecided. Crate-contract "Owns fuel" line corrected.
- **ADP-004 (draft)** — Unicode identifier/security policy: decomposed
  identifiers rejected, confusables unrestricted, NBSP rejected; all pinned by
  `unicode_security.rs`, none changed.
- **ADP-005 (accepted)** — parser recursion bounds (128/64/128) with margin
  analysis, new diagnostic code, cascade control.

## Coverage (LLVM, `docs/testing/coverage.sh`)

- Totals: 52.4% lines / 38.5% functions / 46.2% regions (includes test code,
  bench runner at 0% by design, and third-party deps).
- Workspace src highlights: lexer 96/100, hir 94/100, formatter 94/97, js
  94/80, cli 92/86, sema 89/93, syntax 82/73, vm 84/82, stdlib 84/95.
- Lowest files drove targeted tests: runtime/source error Display, exotic
  placeholder spans. Remaining navigation items (math/prelude error branches,
  disassembler goldens, value fault branches) are listed in
  `docs/testing/gauntlet-gap-matrix.md` — no 100% goal (behavior first).

## Fuzz report

| Target | Budget | Execs | Crashes | Result |
|---|---|---|---|---|
| `lexer_tokens` (libFuzzer) | 60 s + 22 seeds | 115,186 | 0 | clean |
| `frontend_check` (libFuzzer) | ~90 s + 22 seeds + crash replay | 65,801 | 1 (IR panic → fixed, program 23) | 1 fixed |
| `formatter` (libFuzzer) | 120 s + 22 seeds | 215,209 | 0 | clean |
| Deterministic smoke (`fuzz_smoke`, 5 tests) | fixed seeds | — | 0 | clean |

## Mutation report (manual spot-check, `cargo-mutants` deferred)

| Mutant | Result |
|---|---|
| Arity `==` → `>=` (call.rs) | **Survived everything** → wrote `test_call_with_extra_arguments_faults_on_arity` → killed → reverted |
| Safe-int range `..=` → `..` (value.rs) | Killed by `value_properties` → reverted |
| Block depth bound 64 → 1M (parser.rs) | Killed (resource test aborts = guard load-bearing) → reverted |

## Security findings

- Verified protection: no `unsafe` in workspace (`forbid`); transitive `unsafe`
  inventoried (memchr/serde/byteorder/itoa/syn/unicode-normalization — all
  mainstream, zero advisories); audit green; deny green (new `deny.toml`
  policy); hostile bytecode rejected without panic (11 tests); deep input
  terminates with one diagnostic; infinite loop/recursion bounded externally
  (ADP-003 records the missing in-VM budget honestly).
- Verified gap: in-VM fuel/memory/interruption (ADP-003); identifier policy
  (ADP-004).
- Future risk: native/FFI boundaries (none exist) → sanitizers then.

## Differential VM ↔ JS report

- Hand-authored: 23 programs + 19 diagnostics + 3 module cases + 23 examples.
- Generated: 80 AipoSmith programs + 10 metamorphic cases, all VM==JS.
- Divergences found: 3 (journal recovery, signed zero ×2) — all fixed with
  fixtures. Remaining: none.

## Diagnostic accessibility report

- 10 UI goldens (human + JSONL) pin code/severity/message/span/notes/
  suggestions/exit/stdout-discipline; zero ANSI bytes asserted everywhere;
  cascade bounded (nesting = exactly 1 error); spanless module errors covered.
- Suggestion mechanics proven on constructed diagnostics (no producer exists —
  recorded, not invented).
- Rubric (`diagnostic-accessibility-rubric.md`) + manual protocol
  (`cognitive-accessibility-protocol.md`, 10 mistakes) published; sessions not
  yet run (residual).

## Examples catalog

23 examples (01–05 pre-existing, 06–24 new), each with committed `.stdout`,
each verified on VM and Node by `examples.rs`. Detailed per-example table in
the final report (§K).

## Tooling decisions

See `docs/testing/tooling-decisions.md` (adopted: bench runner, LLVM coverage,
cargo-fuzz, audit, deny, Miri, testkit; rejected: Criterion, proptest, AFL++;
deferred: mutants, nextest, sanitizers, Iai). MSRV 1.85 verified.

## Residual risks (explicit)

- Fuzz budgets were minutes, not days; deeper libFuzzer runs live in the
  nightly tier.
- Wall-clock baselines are shared-runner noisy (MAD published alongside).
- Cognitive protocol sessions not yet run.
- `publish = false` added to all crates (deny-driven); publishing to crates.io
  needs a release pass (versions, versions, changelogs per crate).
- Historical counts in old evidence files are historical by policy; current
  counts live in this record.

## Documentation impact

New: `aipo-testkit`, `aipo-bench`, `fuzz/` (+README, targets, seeds),
`deny.toml`, `docs/performance/baseline.md`, `docs/testing/{gap-matrix,
tooling-decisions, ci-tiers, concurrency-audit, rubric, protocol,
coverage.sh}`, ADP-003/004/005, `examples/06–24` + snapshots, programs
21–23, UI goldens, this record. Updated: testing-strategy, crate-contracts
(vm Owns), catalog (`AIPO_PARSE_NESTING_TOO_DEEP`), conformance README
(23/23), `Cargo.toml` repository URL, CHANGELOG (below), PROJECT_STATE,
PRUMO.md.
Contradictions found: stale repo URL (fixed), vm fuel claim (fixed),
gc-arena mentions (fixed earlier), README Prumo link (added earlier).
