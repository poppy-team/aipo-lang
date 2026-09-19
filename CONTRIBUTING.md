# Contributing to Aipo

## Workflow: no code without a Goal

Aipo is governed by Prumo: every change belongs to an explicit, measurable Goal
(`.ai/goals/<phase>/`). Before writing code:

1. Read `ENTRYPOINT.md`, `prumo.json`, `PROJECT_STATE.md` and `docs/PRUMO.md`.
2. Read the active Goal and work with the minimum sufficient context
   (Lean Progressive Context — do not load the whole repo by default).
3. Never weaken acceptance criteria silently.

```bash
prumo validate .
prumo doctor .
prumo goal list
```

## No-invention policy

Semantic decisions not covered by `docs/canon/` must **not** be invented. Open
questions become ADPs under `docs/adp/` with a demonstrating program, the canon
passage (or its absence), and the decision deferred to an explicit goal. The corpus
under `docs/conformance/` is the referee: anything the language claims is either
run there with a committed output or rejected there with a committed diagnostic code.

## Quality gates (all green, always)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace          # needs Node >= 20 for the JS differential suites
cargo doc --workspace --no-deps
```

Plus `prumo validate .` and `prumo doctor .`. Rules enforced by CI and by convention:

- `unsafe_code = "forbid"` workspace-wide; safe Rust only.
- No Rust panic may escape as an Aipo user error — everything is `Result`-shaped;
  the fuzz suites (`fuzz_smoke`) guard this.
- `pub` is opt-in with rustdoc; keep the dependency graph acyclic
  (frontend never depends on backend; `aipo-js` depends on `aipo-ir` only).
- Divergence between the VM and JS backends is a bug until proven otherwise.

## Documentation delta

Code, tests and canonical docs move together. After a behavior change, update the
impacted docs (`docs/conformance/README.md` matrix, `docs/stdlib/mvp-subset.md`,
`docs/reference/cli.md`, diagnostics catalog, crate contracts, `CHANGELOG.md`,
`PROJECT_STATE.md`) and record an evidence file under `docs/evidence/`.
`prumo docs impact <paths…>` helps find what is affected.

## Conformance corpus rules

- `docs/conformance/programs/*.stdout` is the exact output of `aipo run`. A diff in
  a regenerated snapshot is a language change — review it as one, never rewrite
  silently (`AIPO_UPDATE_SNAPSHOTS=1 cargo test -p aipo-cli --test conformance`).
- `diagnostics/*.code` lists one diagnostic code per line; every listed code must
  appear, so failing for the *wrong* reason also fails.
- `formatting/*.expected.aipo` is both formatter output and canonical source.
- New language behavior needs a new fixture proving it on **both** backends.

## License

By contributing, you agree that your contributions are licensed under the
MIT OR Apache-2.0 dual license of this repository.
