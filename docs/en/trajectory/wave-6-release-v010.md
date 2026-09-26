# Wave 6 — Towards Language Release v0.1.0

**Wave 6 (P05)** consolidates the formal product boundary and prepares the first canonical release of the language: **`aipo v0.1.0`**.

---

## Canonical Product Boundary (ADP-008)

In alignment with architectural decision **ADP-008**, release v0.1.0 adopts a **language-first, scoped-boundary** strategy:

- **In Scope for v0.1.0**:
  - The core language: compiler, bytecode format, VM, cooperative async scheduler, contracts, and invariants.
  - Complete standard library with differential VM ↔ JS parity.
  - Unified CLI utility (`aipo run`, `check`, `build`, `disasm`, `fmt`, `package`).
  - Minimalist built-in test runner.
  - Versioned synchronous C ABI (ADP-009).
  - Native Rust embedding API and three thin interoperability proofs in Rust, C, and JavaScript (ADP-010).

- **Out of Scope for v0.1.0 (Post-V1 Roadmap)**:
  - Full game engines and visual authoring editors.
  - Centralized public cloud package registry.
  - Broad web profiles and heavyweight IDE language server extensions.

---

## Versioned Synchronous C ABI (ADP-009)

To enable embedding Aipo seamlessly into hosts written in C, C++, Zig, or any language supporting C FFI, Wave 6 defines a stable C Application Binary Interface (ABI):

- Opaque handle types for VM instances and compiled bytecode chunks.
- Synchronous execution model passing primitives and diagnostic code captures.
- Zero mandatory heap allocations required on caller side.

---

## Thin Interoperability Proofs (ADP-010)

Before declaring v0.1.0 stable, the language verifies real-world embedding viability across three distinct environments via automated integration proofs:

1. **Rust Host Embedding**: A native Rust application initializing the VM, injecting custom native functions, and running Aipo scripts.
2. **C Host Integration**: A compiled C binary linking against the Aipo static/dynamic library and exchanging primitive data safely.
3. **JavaScript / Node.js Runtime**: A Node.js application importing modules emitted by `aipo-js` and executing the runtime shim alongside ecosystem libraries.
