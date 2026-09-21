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

## Exit gate (evidence required)

- Headless Poppy demo deterministic across runs (seeded) on VM; ECS command
  buffer, stale-handle and escape fixtures green with committed codes.
- `time.now`/`monotonic` gated by `clock` capability; denial faults.
- Full workspace gates green; `docs/evidence/P03-G*` recorded.
