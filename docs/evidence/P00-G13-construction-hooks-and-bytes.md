# Evidence — P00-G13 / MVP Construction Hooks & `Bytes`

**Goal:** `P00-G13` — MVP contract closure: `init`, `invariant`, runtime contracts and `Bytes` construction
**Phase:** P00 (Foundation) · **Recorded:** 2026-09-15
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.

## Why this goal existed

The S11 conformance corpus (`docs/evidence/P00-G12-conformance-and-mvp-gate.md`) found four places
where canon and the implementation disagreed, and recorded the MVP gate as **partial** rather than
closing it. This goal resolves three of them from canon and re-delegates the fourth.

| Gap | Decision source | Outcome |
|---|---|---|
| G1 — `init` not invoked by `Type{...}` | Canonical Syntax: `Type{...}` uses `init` when it exists; the hook is not a dot-call | **closed** |
| G2 — `invariant()` not evaluated | Canonical Syntax: verified at the end of construction; lines combined by `and` | **closed (at construction)** |
| G4 — `Bytes` had no construction form | Language Reference §6: `let data = Bytes(32)` — a managed mutable block | **closed** |
| G2b — invariant at mutation points | Canonical Syntax: `candidate → apply → verify → commit` | deferred to `P00-G14` |
| G3 — signature contracts at runtime | Wave 1 plan: "runtime check at boundaries" | deferred to `P00-G14` |

The error channel for both contract violations was already decided by the repo's own MVP error
model (`docs/stdlib/mvp-subset.md`): **contract violations are runtime faults**, not recoverable
`Failure`s. No new semantics had to be invented.

## Implementation

| Stage | Change |
|---|---|
| `aipo-syntax` | `parse_init_hook` injects the implicit `self!` receiver when the author omits it, so canon's `init(id, name)` and the explicit `init(self!, id)` form lower identically |
| `aipo-hir` | `invariant()` lowers to a `self`-taking predicate: the hook's lines become one `and` chain returned by `<Type>.invariant` |
| `aipo-ir` | `init` hooks are registered as construction signatures (declared parameters only); `Type{...}` emits `BuildStruct(defer_fixed)` → `Type.init` call → `Pop` → invariant check → `SealStruct`; `init` returns the instance it was handed |
| `aipo-bytecode` | `BuildStruct` carries a `defer_fixed` flag; new opcodes `SealStruct` and `AssertInvariant` (emitter, verifier with name-index bounds checks, disassembler) |
| `aipo-vm` | `StructInstance::under_construction` keeps `fixed` fields mutable until `seal`; `BuildStruct` defers both the `fixed` set and the invariant; `SealStruct` restores the set; `AssertInvariant` turns a `false` predicate into `VmFault::InvariantViolation`; `Bytes(count)` conversion |

## Gate: `fmt` / `check` / `clippy`

```
$ cargo fmt --all -- --check
FMT=0

$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)

$ cargo clippy --workspace --all-targets -- -D warnings
(zero warnings)
```

## Gate: `test`

```
$ cargo test --workspace
passed: 137  failed: 0
```

New certification:

| Fixture | Proves |
|---|---|
| `docs/conformance/programs/13_init_and_invariant.aipo` | canon `init` with defaults, `fixed` assignment during construction, default referencing an earlier parameter, invariant verified at the end of construction |
| `docs/conformance/diagnostics/11_runtime_invariant_violation.aipo` | a violated invariant fails with `AIPO_RT_TYPE_MISMATCH` and the instance is never published |
| `docs/conformance/programs/12_bytes.aipo` | `Bytes(4)` size, byte indexing, `len`, slicing, `Bytes(0)` |
| `crates/aipo-stdlib/tests/stdlib_tests.rs::test_convert_bytes` | `Bytes(count)` shape, zero-fill, `Bytes(0)`, negative/oversized `Failure`, non-`Int` fault |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
(no warnings)
```

## Gate: `documentation_impact`

- `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` — G1, G2 and G4 marked resolved with
  the implementation decision recorded; G2b and G3 rewritten as the next goal's criteria.
- `docs/stdlib/mvp-subset.md` — the `Bytes` section now documents the canonical `Bytes(count)` form
  and the provisional allocation cap.
- `docs/conformance/README.md` — snapshot matrix extended; the "Verified gaps" section now states
  which gaps closed and which remain.
- `CHANGELOG.md`, `PROJECT_STATE.md`.

## Known limitations (recorded, not hidden)

- `invariant()` is verified at the end of construction only. Mutating a field after publication
  does not re-evaluate the predicate yet — gap **G2b**, assigned to `P00-G14`.
- `init` bodies that execute an explicit `return` leave the construction value as that return
  rather than the instance. Canon's `init` is a hook without a value, so this is only reachable by
  writing an unusual body; the normal path returns the sealed instance.
- `BYTES_MAX_ALLOCATION` (64 MiB) is a provisional implementation guard, not a canon limit.
