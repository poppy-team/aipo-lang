# aipo-poppy

`aipo-poppy` is the Poppy Game Engine reference profile and deterministic headless adapter for Aipo (Wave 4, `P03-G02`).

## Architecture & Guarantees

- **AHS Host Surface (`poppy_schema`)**: The Poppy surface is described strictly as data via the Aipo Host Schema (AHS). It exposes modules, types (`Vec2`, `Transform`), handles (`Entity`), functions (`spawn`, `despawn`, `query`, `get_position`, `set_position`, `get_velocity`, `set_velocity`, `random_float`, `random_int`, `step`, `digest`), and capabilities (`poppy.ecs`, `poppy.random`).
- **Entity Generational Handles (`Handle`)**: Entities are addressed via opaque generational handles from `aipo-host`. A despawned entity advances the generation, ensuring stale handles fault with `AIPO_RT_STALE_HANDLE` rather than encountering use-after-free or accessing reused slots.
- **Safe-Point ECS Command Buffer (`CommandBuffer`)**: Structural mutations (spawns, despawns, component updates) are recorded into a command buffer and applied only at explicit safe points between system updates or during `step()`. This prevents query iterator invalidation during game loop traversals.
- **Deterministic Simulation**: A fixed-time-step physics integrator and a seeded PRNG (`PoppyRng`) drive the headless game simulation. State digests (`digest()`) computed via 64-bit FNV-1a hashing guarantee 100% bit-exact reproducibility across repeated runs given the same initial seed.
- **Capability Sandbox**: All Poppy services require the `poppy` capability tree (`poppy.ecs`, `poppy.random`). An execution without these grants fails immediately with `AIPO_RT_CAPABILITY_DENIED`.
- **Clean Architecture & Safety**: `#![forbid(unsafe_code)]`. Bytecode, IR and compiler never depend on Poppy concepts; Poppy adapts onto the host ABI.

## Testing

Run unit and integration tests:
```bash
cargo test -p aipo-poppy
```
This exercises AHS schema validation, PRNG determinism, command buffer deferral, and the end-to-end headless game demo fixture (`tests/headless_demo.rs`).
