# CI Tiers

**Status:** normative for CI design (process level)
**Scope:** which checks run where; no expensive test in every PR
**Related:** `docs/development/testing-strategy.md` (blocking gates),
`docs/testing/tooling-decisions.md`

> Note: this repository currently has no remote CI configuration; the tiers
> below are the design to implement when CI lands. Names are
> repository-appropriate: `fast`, `nightly`, `perf`.

## Tier `fast` (every PR)

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets   # includes conformance, differential (Node >= 20), fuzz smoke, UI goldens, examples
cargo doc --workspace --no-deps
cargo build -p aipo-bench             # benchmark compilation only
prumo validate . && prumo doctor .
```

Rationale: everything here completes in ~1–2 minutes and is deterministic
(fixed seeds; no wall-clock assertions).

## Tier `nightly` (scheduled)

```bash
bash docs/testing/coverage.sh --report coverage.txt
cargo +nightly fuzz run lexer_tokens -- -max_total_time=600
cargo +nightly fuzz run frontend_check -- -max_total_time=600
cargo +nightly fuzz run formatter -- -max_total_time=600
cargo +nightly miri test -p aipo-vm
cargo audit
cargo deny check
cargo test --workspace --all-targets  # extended: generated-large seeds, resource suite
```

New crashes become minimized regression fixtures before the tier goes green
again (regression policy).

## Tier `perf` (dedicated runner only)

```bash
cargo run --release -p aipo-bench -- --json baseline-candidate.json
# compare against docs/performance/baseline.md; investigate >20% median
# regressions on same hardware before merging perf-sensitive changes
```

Wall-clock gates never block on shared runners (noise). Blocking performance
gates additionally require: stable hardware, three historical baselines, and
an explicit accepted threshold per workload.
