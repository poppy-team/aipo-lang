# What is Aipo?

**Aipo** is a modern general-purpose programming language featuring dynamic and strong typing, paired with native signature contracts and structural invariants.

Engineered with a philosophy of **clarity, robustness, and predictability**, Aipo was built from the ground up in Rust using Clean Architecture boundaries.

---

## Core Pillars

### 1. Dynamic and Strong Typing

In Aipo, values have concrete types and the runtime never performs arbitrary or hidden conversions between incompatible types:

```aipo
let x = "42"
let y = 10

// Runtime type fault:
// Operator '+' does not concatenate String and Int implicitly
let z = x + y // Explicit failure!
```

Converting or concatenating values requires clear programmer intent.

### 2. Structural Invariants & Contracts

While traditional languages require scattering manual `assert` calls across every method, Aipo introduces `invariant()` directly inside struct declarations:

```aipo
struct Temperature {
  var celsius: Float,

  invariant() {
    self.celsius >= -273.15 // Absolute zero boundary
  }
}
```

Any mutation that violates an invariant is intercepted at the boundary. Inside `attempt ... recover` transaction blocks, modifications are rolled back atomically to their previous state.

### 3. Deterministic Concurrency

Aipo's concurrency model is founded on **cooperative fibers and virtual time**. Async calls utilize `async fn` and high-level combinators (`task.spawn`, `task.sleep`, `task.all`, `task.race`), orchestrated by a deterministic scheduler that enables 100% reproducible tests.

### 4. Dual Target: Native VM and JavaScript

The Aipo compiler targets two first-class runtimes:
1. **Rust Bytecode VM**: Fast execution, deterministic `.aibc` binary serialization, and instruction disassembly with mapped line/column metadata.
2. **JavaScript Backend (`aipo-js`)**: Direct transpilation to modern ES2022 with a modular runtime shim, guaranteeing bit-for-bit differential parity.

---

## Quick Comparison

| Feature | Aipo | Python | Lua | JavaScript |
| :--- | :--- | :--- | :--- | :--- |
| **Typing** | Dynamic & Strong | Dynamic & Strong | Dynamic & Weak | Dynamic & Weak |
| **Native Invariants** | Yes (`invariant()`) | No (manual) | No | No |
| **Transactional Rollback** | Yes (`attempt`) | No | No | No |
| **Native Async** | Yes (Deterministic) | Yes (`asyncio`) | Coroutines | Yes (Event Loop) |
| **Package Panning** | Commit SHA + Offline | Pip / Virtualenv | External (Luarocks) | NPM / Node Modules |
| **Implementation** | Rust (Compiler + VM) | C / C++ | ANSI C | C++ (V8) / Rust (Deno) |
