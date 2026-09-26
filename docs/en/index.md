---
layout: home

hero:
  name: Aipo
  text: Dynamic, Strong & Concurrent Language with Optional Contracts
  tagline: Pure Rust-first pipeline with dedicated compiler, deterministic bytecode VM, cooperative async scheduler, JavaScript backend with differential parity, and Prumo CLI governance.
  image:
    src: /assets/logo.svg
    alt: Aipo Language Logo
  actions:
    - theme: brand
      text: Get Started (5 min)
      link: /en/getting-started/first-program
    - theme: alt
      text: Engineering Trajectory
      link: /en/trajectory/
    - theme: alt
      text: Language Manual
      link: /en/manual/
    - theme: alt
      text: GitHub Repository
      link: https://github.com/poppy-team/aipo-lang

features:
  - icon: ⚡
    title: Native Rust Pipeline
    details: Written from scratch in modern Rust. UTF-8 boundary-safe lexer, resilient recovery parser, HIR, strict SEMA with contract inference, and compact .aibc bytecode.
    link: /en/architecture/compiler-frontend
  - icon: 🛡️
    title: Contracts & Transactional Invariants
    details: Static and runtime verification. Structural invariant() hooks with atomic rollback on failure inside attempt blocks, signature assertion, and interface conformance.
    link: /en/manual/interfaces-and-contracts
  - icon: 🔄
    title: Async Concurrency & Virtual Time
    details: Native async fn and await do syntax, deterministic async combinators (task.spawn, task.all, task.race), cooperative scheduler, and await cycle detection.
    link: /en/manual/async-and-concurrency
  - icon: 🌐
    title: VM ↔ JavaScript Differential Parity
    details: aipo-js emitter with versioned modular runtime shim, producing clean ES2022 code for Node.js and browsers with bit-for-bit identical outputs and diagnostics.
    link: /en/architecture/js-emitter
  - icon: 🔒
    title: Secure Host ABI & Poppy Simulation
    details: Sandboxing with deny-by-default capabilities, generational handles immune to use-after-free, strict scope escape prevention, and deterministic headless ECS simulation.
    link: /en/architecture/host-abi
  - icon: 📦
    title: Hermetic & Offline Package Manager
    details: aipo.toml manifest, deterministic lockfiles, remote GitHub dependencies pinned by full commit SHA with SHA-256 digest validation, atomic local cache, and pruning.
    link: /en/manual/packages-and-modules
---

<div class="vp-doc">

## A Modern Language Built on Engineering Discipline

**Aipo** was engineered to offer clean, expressive syntax without sacrificing technical rigor and predictability. Instead of dangerous implicit coercions (such as `"1" + 2 == "12"`), Aipo pairs dynamic, strong typing with **optional structural contracts**, **transactional failure recovery**, and a **deterministic async system**.

```aipo
# Canonical Aipo example: structs, invariants, and async
struct Account {
    id
    var balance = 0.0
    created_at
}

impl Account {
    fn init(id, balance = 0.0, created_at = 0) {
        self.id = id
        self.balance = balance
        self.created_at = created_at
    }

    invariant {
        self.balance >= 0.0
    }

    fn transfer(var self, target: Account, amount: Float) {
        if amount <= 0.0 {
            return fail("Amount must be positive")
        }

        # If any invariant fails, all mutations suffer atomic rollback
        attempt {
            self.balance -= amount
            target.balance += amount
        } failed err {
            return fail(f"Transfer cancelled: {err.message}")
        }
    }
}

# Automatic structural subtyping: Account satisfies Payable directly
interface Payable {
    fn transfer(var self, target: Account, amount: Float)
}

async fn process_payment(acc: Payable, amount: Float) {
    let timer = task.sleep(100)
    await do {
        timer
        io.println("Payment processed successfully")
    }
}
```

::: tip 💡 Getting started in 3 simple steps
1. **Compile or install the CLI**: `cargo build --release -p aipo-cli` (or install the `aipo` binary)
2. **Run your first program**: Follow the 5-minute tutorial in [Your First Program](/en/getting-started/first-program)
3. **Explore the journey**: Discover all implementation waves in the [Engineering Trajectory](/en/trajectory/)
:::

---

## The Engineering Trajectory

Unlike experimental languages developed without traceability, Aipo was built under the **Evidence-Based Engineering** protocol governed by the **Prumo CLI**. Every compiler phase, runtime feature, and ecosystem tool is certified by automated test suites, comparative benchmarks, and canonical specs:

| Wave / Phase | Core Focus | Deliverables & Status |
| :--- | :--- | :--- |
| **Wave 0 (MVP)** | Slices S1 to S11 | Lexer, Parser, HIR, Sema, IR, Bytecode, VM, CLI, Conformance *(100% Completed)* |
| **Wave 1** | Contracts & Integrity | Structural invariants, rollback with `attempt`, interfaces & local functions *(100% Completed)* |
| **Wave 2 (P01)** | JS Backend & Fuzzing | `aipo-js` emitter, differential parity, property suites & `cargo deny` *(100% Completed)* |
| **Wave 3 (P02)** | Rich Types & Async | `Set`, `Sequence`, `Bytes`, `async fn`, `await do`, deterministic scheduler *(100% Completed)* |
| **Wave 4 (P03)** | Host ABI & Poppy | Deny-by-default capabilities, generational handles, and seeded simulation *(100% Completed)* |
| **Wave 5 (P04)** | Package Manager | Commit-SHA pinned GitHub dependencies, SHA-256 digest validation & offline cache *(100% Completed)* |
| **Wave 6 (P05)** | Release v0.1.0 | Language-first product boundary (ADP-008), synchronous C ABI (ADP-009) & thin proofs *(In Progress)* |

</div>
