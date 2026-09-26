# Getting Started with Aipo

Welcome to the official documentation of **Aipo**, a dynamic, strongly typed programming language with optional contracts, built from scratch in Rust.

---

## Suggested Pathway

If you are new to the language, we recommend the following path:

1. **[What is Aipo?](/en/getting-started/what-is-aipo)**  
   Learn about our design philosophy, why we reject silent implicit coercions, and how Aipo compares to Lua, Python, Wren, and TypeScript.

2. **[Installation & Setup](/en/getting-started/installation)**  
   How to build the `aipo` CLI utility from source using Cargo or run precompiled binaries on Linux, macOS, and Windows.

3. **[Your First Program in 5 Minutes](/en/getting-started/first-program)**  
   A hands-on walkthrough with examples of syntax, structures, invariants, and immediate execution in the VM and JS runtime.

4. **[CLI Overview (`aipo`)](/en/getting-started/cli-overview)**  
   Explore command-line operations: `run`, `check`, `build`, `disasm`, `fmt`, and `package`.

---

## Why Aipo?

- **Zero Implicit Coercions**: Eliminates subtle type bugs common in JavaScript and Python (`"10" + 2` is a type fault, never `"102"` or `12`).
- **Contracts & Invariants**: Express semantic domain rules directly within structures (`invariant()`), guaranteed by automatic mutation rollback inside `attempt` blocks.
- **Deterministic Cooperative Concurrency**: First-class support for `async fn` and task combinators (`task.*`), backed by a virtual-time scheduler that guarantees zero flaky tests.
- **Hermetic Ecosystem**: Dependencies pinned by commit SHA with SHA-256 cryptographic hashes and 100% offline reproducibility.
