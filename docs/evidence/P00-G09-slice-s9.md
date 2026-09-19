# Evidence — P00-G09 / Slice S9 (Runtime & Minimum Stdlib)

**Goal:** `P00-G09` — Runtime & Minimum Stdlib: Module registry, initialization order, Prelude V1, and native stdlib modules
**Phase:** P00 (Foundation) · **Slice:** S9 · **Recorded:** 2026-09-15
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.

## Gate: `fmt`

```
$ cargo fmt --all -- --check
(no output, exit 0)
```

## Gate: `check`

```
$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Gate: `clippy` (workspace lints deny warnings)

```
$ cargo clippy --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
(no warnings; workspace lints set `all/correctness/suspicious/complexity/perf/style = deny`)
```

## Gate: `test`

```
$ cargo test --workspace
total_passed=109 total_failed=0
```

Per crate:

| Crate | Tests |
|---|---|
| `aipo-stdlib` | 30 passed, 0 failed |
| `aipo-runtime` | 10 passed, 0 failed |
| `aipo-vm` | 24 passed, 0 failed |

`aipo-stdlib` coverage (`crates/aipo-stdlib/tests/stdlib_tests.rs`, 30 tests):
Prelude V1 (`len`, `copy`, `same`, `some`, `fail`), core-type conversions
(`Int`, `Float`, `Byte`, `String`), `math` (`abs`, `min`, `max`, `floor`, `ceil`, `round`
half-away-from-zero, `truncate`, `sqrt`, `pow`, `clamp`), `string` (`len`, `byte_len`,
`contains`, `starts_with`, `ends_with`, `find`, `lower`, `upper`, `capitalize`, `reverse`,
`trim`, `split`, `join`, `replace`, `slice`, `format`), `io` (capture sink), canon argument
rules (empty `split`/`replace` patterns, non-`String` `join` elements, preserved empty
fields), `NativeRegistry` surface coverage, and two end-to-end VM executions calling
stdlib natives through bytecode.

`aipo-runtime` coverage (`crates/aipo-runtime/tests/graph_tests.rs`, 10 tests): empty graph,
single module, linear chain, lexicographic tie-break, diamond dependency, missing dependency,
direct cycle, indirect cycle, module lifecycle states, native registry operations.

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
(no warnings; `missing_docs` is enabled at workspace level in every crate)
```

## Governance gates

```
$ prumo validate
Prumo validation passed.

$ prumo doctor
Prumo Doctor: all checks passed cleanly.

$ prumo docs verify
{"files":68,"checks":["authority","managed-regions","documentation-links","claim-drift","terminology","version-policy"],"findings":[],"ok":true}

$ prumo docs authority        # {"files":68,"canonical":68,"findings":[],"ok":true}
$ prumo docs contradictions   # []

$ prumo docs plan --goal P00-G09 --changed crates/aipo-stdlib docs/stdlib docs/adp docs/security --reconcile
{"predicted":["security.trust"],"actual":["security.trust"],"missing":[],"unpredicted":[],"drift_ratio":0,"findings":[]}

$ prumo goal list             # P00-G09 DONE, diagnostics [], warnings []
$ prumo report add            # TR-002 recorded in .prumo/history/project-intelligence.json
```

Documentation obligations predicted for the touched paths were `security.trust`; the actual delta
was the same contract, so the postflight reconcile reports zero drift.

## Dependency approval

`unicode-normalization` 0.1.25 contains `unsafe` (`char::from_u32_unchecked`, Hangul
decomposition). Approval was requested and granted explicitly during this slice; the
exception is recorded in `docs/security/security-contract.md`. `unicode-segmentation`
1.13.3 (`#![deny(unsafe_code)]`) and `tinyvec` 1.13.3 (`#![forbid(unsafe_code)]`) were
verified unsafe-free.

## Documentation delta

| Artifact | Change |
|---|---|
| `docs/stdlib/mvp-subset.md` | new — implemented Prelude V1 / `math` / `string` / `io` surface, error model, deferred surface |
| `docs/adp/ADP-001-byte-and-core-types-as-values.md` | new — open questions: `Byte` runtime kind, first-class type values, inverted `clamp` bounds, `slice` saturation, NFC boundaries |
| `docs/crates/crate-contracts.md` | `aipo-stdlib` contract now points at the implemented surface and ADP-001 |
| `docs/security/security-contract.md` | approved dependency exception table + unsafe scan requirement |
| `docs/PRUMO.md` | router links to crate contracts, wave plan, authority map, stdlib subset, ADP-001 |
| `crates/aipo-stdlib/README.md` | surface, invariants and error-model summary |
| `crates/aipo-runtime/README.md` | unchanged (accurate for this slice) |
| `PROJECT_STATE.md`, `CHANGELOG.md` | slice S9 recorded, next action set to S10 |
