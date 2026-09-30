# Evidence — P00-G12 / Slice S11 (Conformance Hardening & MVP Gate)

**Goal:** `P00-G12` — Conformance hardening: fixtures, snapshots, gauntlet and MVP gate
**Phase:** P00 (Foundation) · **Slice:** S11 · **Recorded:** 2026-09-15
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.
The corpus layout and snapshot matrix live in `docs/conformance/README.md`; this record is the
score, not the specification.

## Deliverable

The slice turns "the MVP works" into a claim a command can falsify. Everything the MVP subset
claims is either executed with a committed output, or rejected with a committed diagnostic code:

| Kind | Count | Location |
|---|---|---|
| Runnable programs + stdout snapshots | 11 + 11 | `docs/conformance/programs/` |
| Diagnostic fixtures + expected codes | 10 + 10 | `docs/conformance/diagnostics/` |
| Formatter golden pairs | 8 + 8 | `docs/conformance/formatting/` |
| Module cases | 3 | `docs/conformance/modules/` |

Snapshots are observations and are regenerated only on request
(`AIPO_UPDATE_SNAPSHOTS=1`); diagnostic codes, formatter expectations and module fixtures are
hand-authored because they are specifications.

## Gauntlet rubric

| Gate | Weight | Command | Result |
|---|---|---|---|
| Executable MVP | 30% | `cargo test -p aipo-cli --test conformance` | **pass** — 13/13, all program and module fixtures match committed stdout |
| Diagnostic discipline | 20% | same suite, failure tests | **pass** — every fixture fails with its committed code, including the guard that a parse fixture reports an `AIPO_PARSE_*` code |
| Determinism | 20% | `cargo test -p aipo-formatter` | **pass** — 11/11; formatter idempotent, canonical sources report no drift, `fmt --check` never rewrites |
| Robustness | 15% | `cargo test -p aipo-cli --test fuzz_smoke` | **pass** — 3/3; random bytes, mutated programs and truncated programs never panic the pipeline |
| Repository gates | 15% | `cargo fmt --check`, `clippy -D warnings`, `test --workspace`, `doc` | **pass** — all four green |

Command output:

```
$ cargo test -p aipo-cli --test conformance
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p aipo-cli --test fuzz_smoke
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p aipo-formatter
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Gate: `fmt` / `check` / `clippy` / `test` / `doc`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)

$ cargo clippy --workspace --all-targets -- -D warnings
(no output, exit 0)

$ cargo test --workspace
passed: 136  failed: 0

$ cargo doc --workspace --no-deps
   Generated /home/raillen/Documentos/Projetos/aipo-lang/target/doc/aipo_ast/index.html and 14 other files
```

## MVP exit gate

`docs/waves/wave-1-mvp.md` lists five exit conditions. Status against each:

| Exit condition | Status | Evidence |
|---|---|---|
| `aipo run` executes real small programs | **met** | `programs/*` snapshots, `modules/*` cases |
| `aipo check` reports diagnostics; `aipo fmt` idempotent | **met** | `diagnostics/*` fixtures; formatter golden + idempotency tests |
| pass/fail fixtures green; snapshots committed; integration tests green | **met** | conformance suite 13/13 |
| No Rust panic escapes as user error (fuzz smoke) | **met** | `fuzz_smoke` 3/3 |
| CI: fmt/clippy/test/doc green; `prumo validate` and `prumo doctor` green; docs delta resolved | **met** | gates above; `prumo` outputs below |

```
$ prumo validate
Prumo validation passed.

$ prumo doctor
Prumo Doctor: all checks passed cleanly.
```

## Verified gaps (gate is partial, and says why)

The corpus found four places where canon and the implementation still differ. Exercising them is
the point of the corpus — the S11 gate does not hide them. Each is recorded in
`docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` with a minimal demonstrating
program, and each is carried as an acceptance criterion of `P00-G13`:

| Gap | Today | Canon requires |
|---|---|---|
| G1 `init` not invoked by `Type{...}` | construction only fills named fields | construction runs `init` when declared |
| G2 `invariant()` not evaluated | instance published unchecked | invariants hold on publication |
| G3 signature contracts not checked at runtime | value crosses the boundary unchecked | runtime check at boundaries |
| G4 `Bytes` has no construction form | `Bytes([1,2,3])` raises `AIPO_RT_NOT_CALLABLE` | construction and indexing are in scope |

These are **not** regressions and **not** failures of this goal's acceptance criteria: S11's
deliverable is the corpus and the honest gate, and it delivers both. They are the reason the
gate is recorded as **partial** rather than **closed**.

## Documentation delta

| Artifact | Change |
|---|---|
| `docs/conformance/README.md` | new — corpus layout, fixture invariants, snapshot matrix, regeneration, gauntlet rubric |
| `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` | new — the four verified gaps above, with demonstrating programs |
| `docs/evidence/P00-G11-cli-and-formatter.md` | records the S10 deliverables this slice scores |
| `docs/evidence/P00-G10-backend-completion.md` | records the backend gaps this slice exercises |
| `.ai/goals/P00-G13.goal.json` | successor goal for the four gaps |
| `CHANGELOG.md`, `PROJECT_STATE.md` | slice S11 recorded, next action set to S12 |
