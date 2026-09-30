# Wave 4 — Host ABI + Poppy

**Status:** normative (process level)
**Authority:** canon Waves model (Wave 4), Fechamento §7/§8/§11, Poppy Pivot
(host profiles, AHS, determinism)
**Scope:** `aipo-host` (ABI contracts, capabilities, handles), `aipo-poppy`
(ECS scopes, command buffer, behaviors/events), clock capability
**Slices:** P03-G01 (host ABI) → P03-G02 (Poppy adapter + demo)

## Objective

Prove the host-neutral ABI: capabilities deny-by-default, generational handles
that never use-after-free, AHS describing host surface as data, scoped ECS
mutation applied at safe points, deterministic headless game fixture.

## Non-negotiable rules (from canon)

- Never expose engine internals, Rust references or lifetimes to scripts.
- Host values copy by value; external identity belongs to the host.
- Stale handles never use-after-free (`none`/Failure per contract).
- Scoped bindings cannot escape the callback (enforced at heap-publication points).
- No `behavior`/`component`/`system` keywords: Behavior is a plain interface.
- Deterministic: fixed tick, seeded RNG, command-buffer ordering, digest/replay.

## Progress — P03-G01 (host ABI)

- Delivered: `aipo-host` (capabilities, generational handles, host values, AHS model),
  VM adoption, clock capability on both backends, scoped-escape enforcement at the
  heap-publication points.
- Delivered: `--ahs=<file>` on `aipo run`/`aipo check`. The AHS is read as data, validated, and
  handed to the checker, which resolves host modules and checks calls against their described
  signatures (unknown member, arity with optional parameters, named arguments, literal contract
  violations). The surface is passed explicitly per compilation, with no process-global AHS;
  Rust embedders load it through `load_host_surface` and pass it to `compile_file` or
  `analyze_with_surface`. Local shadowing and struct method checks remain intact, and Wasm
  runs the same semantic checks (runtime host imports are not installed). Fixtures live
  in `docs/conformance/ahs/`, tests in `crates/aipo-cli/tests/host_surface.rs`.
- Not yet delivered: a CLI path that reaches the runtime host faults (`AIPO_RT_CAPABILITY_DENIED`,
  `AIPO_RT_STALE_HANDLE`, `AIPO_RT_SCOPE_ESCAPE`) from a `.aipo` program. They are covered by Rust
  tests only; `--ahs` does not install host implementations or grant capabilities. P03-G01
  stays open until that harness exists.

## Exit gate (evidence required)

- Headless Poppy demo deterministic across runs (seeded) on VM; ECS command
  buffer, stale-handle and escape fixtures green with committed codes.
- `time.now`/`monotonic` gated by `clock` capability; denial faults.
- Full workspace gates green; `docs/evidence/P03-G*` recorded.
