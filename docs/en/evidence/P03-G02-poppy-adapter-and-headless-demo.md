# Evidence — P03-G02 / Poppy Adapter and Deterministic Headless Demo

**Goal:** `P03-G02` — Implement `aipo-poppy` crate as the Poppy Game Engine adapter over `aipo-host` (ECS scopes, command buffer, behaviors/events, deterministic simulation) and prove it with a deterministic headless demo game fixture.
**Phase:** P03 (Wave 4 — Host ABI + Poppy) · **Recorded:** 2026-09-21
**Environment:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (all executed, all green)

```
$ cargo fmt --all -- --check                                           # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings                # 0 warnings
$ cargo test --workspace --all-targets                                 # 391 passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 warnings
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-cli --test conformance                            # programs 1–28, diagnostics 1–29
$ cargo test -p aipo-poppy                                             # 14 passed (11 unit + 3 integration)
```

## Acceptance Criteria Verification

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | Crate `aipo-poppy` with `#![forbid(unsafe_code)]` over `aipo-host` | ✅ | `crates/aipo-poppy/` created, `#![forbid(unsafe_code)]`, imports `aipo-host`, clean workspace layer |
| 2 | Poppy host surface described as data via AHS (`HostSchema`) under `poppy` capability tree | ✅ | `poppy_schema()` in `schema.rs` exposes types (`Vec2`, `Transform`), handle `Entity`, functions (`spawn`, `despawn`, `query`, `get_position`, `set_position`, `get_velocity`, `set_velocity`, `random_float`, `random_int`, `step`, `digest`), requiring `poppy.ecs` and `poppy.random` |
| 3 | Entity identities use generational `Handle`s from `aipo-host` with stale handle detection | ✅ | Entities stored in `HandleTable<EntityRecord>`; post-safe-point access to despawned entities faults with `AIPO_RT_STALE_HANDLE` |
| 4 | Structural ECS mutations recorded in command buffer, deferred to safe points | ✅ | `CommandBuffer` queues `Spawn`, `Despawn`, `SetPosition`, `SetVelocity`; applied in `World::apply_deferred()` during `step()` |
| 5 | Behavior lifecycle adheres to plain interface semantics without language keywords | ✅ | Behavior logic in game script operates on regular functions/methods, inspecting entities and issuing commands |
| 6 | Headless Poppy simulation is completely deterministic (fixed tick, seeded PRNG, command buffer ordering, identical digests across runs) | ✅ | `World::digest()` produces 64-bit FNV-1a state digest over sorted entity slots, components, tick, and RNG state; tests verify bit-exact digest matching across runs |
| 7 | Executable headless demo fixture verifies deterministic multi-tick simulation and command buffer execution | ✅ | `crates/aipo-poppy/tests/headless_demo.rs` compiles and executes multi-tick game script, asserting digest identity across runs with same seed, digest variation with different seeds, stale handle faults, and capability denial |
| 8 | cargo fmt, clippy, test, doc all green | ✅ | See gates above |

## Summary of Implementation

| Component | Responsibility |
|---|---|
| **`schema.rs`** | Canonical `poppy_schema()` implementing the Aipo Host Schema (AHS) for Poppy. Validates without errors against `HostSchema::validate()`. |
| **`prng.rs`** | `PoppyRng`: deterministic xorshift64* pseudo-random number generator with float/int generation methods. |
| **`commands.rs`** | `CommandBuffer`: queues structural mutations (`Spawn`, `Despawn`, `SetPosition`, `SetVelocity`) to defer entity table modifications until safe points. |
| **`world.rs`** | `World`: entity storage backed by generational `HandleTable`, query filter by tag, velocity integration, and deterministic 64-bit FNV-1a state digest computation. |
| **`simulation.rs`** | `Simulation`: fixed-rate game loop coordinator advancing ticks, physics, command flushes, and digest calculation. |
| **`adapter.rs`** | VM integration: provides `poppy` module in `Vm.globals`, gating operations behind `poppy.ecs` and `poppy.random` capabilities, translating entity handles to `Value::HostHandle`. |
| **`tests/headless_demo.rs`** | End-to-end integration test driving a 10-tick game simulation on the Aipo VM, verifying determinism, stale handle detection, and capability enforcement. |

## Test Inventory & Verification

Unit and integration tests:

- `crates/aipo-poppy/src/schema.rs` — schema clean validation, capability declaration.
- `crates/aipo-poppy/src/prng.rs` — determinism test, float range `[0.0, 1.0)`.
- `crates/aipo-poppy/src/commands.rs` — command buffer FIFO queuing and draining.
- `crates/aipo-poppy/src/world.rs` — entity spawn, tag query, deferred despawn, digest determinism.
- `crates/aipo-poppy/src/simulation.rs` — 60-tick simulation reproducibility across instances.
- `crates/aipo-poppy/src/adapter.rs` — capability denial fault, granted execution, safe-point stale handle fault.
- `crates/aipo-poppy/tests/headless_demo.rs` — full VM pipeline execution of game script: identical digests with seed 1337, different digests with seed 9999, stale handle fault post-despawn, capability denial fault on ungranted environment.
