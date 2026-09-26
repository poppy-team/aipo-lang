# Wave 4 — Host ABI, Sandboxing & Poppy Engine

**Wave 4** engineered Aipo's security boundary and host interoperability substrate, formalizing the **Host ABI (`aipo-host`)** and validating it in practice against the **Poppy Headless Simulation Engine (`aipo-poppy`)**.

---

## Achieved Milestones

### 1. `aipo-host` Crate & Host Schema (AHS)
- Formalized communication protocol bridging the language runtime and host Rust applications:
  - **Deny-by-Default Capabilities**: No system resource (clock, I/O, network, filesystem) is accessible unless the host explicitly grants granular capability permissions (`CapabilitySet`).
  - **Generational Handles (`HandleTable`)**: Host-managed resources exposed to Aipo scripts are identified via generational handles with strict generation and index verification, completely eliminating *use-after-free* hazards.

### 2. Strict Scope Escape Prevention
- Deep verification across all 6 VM publication points (`SetGlobal`, `Return`, `SetField`, `SetIndex`, `BuildList`, `BuildDict`):
  - Guarantees that handles and values bound to a confined lifecycle cannot survive beyond their active scope, raising the deterministic runtime fault `AIPO_RT_SCOPE_ESCAPE`.

### 3. Poppy Adapter & Deterministic Headless Simulation (`aipo-poppy`)
- Language integration with an ECS (Entity-Component-System) simulation runtime:
  - Securely exposes the `poppy` module guarded under the `poppy.*` capability tree.
  - Deferred structural mutations buffered in a transactional `CommandBuffer` evaluated exclusively at engine safe points, preserving iteration invariants.
  - Seedable pseudo-random number generator (`PoppyRng`), guaranteeing 100% mathematical reproducibility across runs.
  - End-to-end integration fixture running headless entity simulations, proving Host ABI design stability.
