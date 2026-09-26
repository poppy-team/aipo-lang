# Architecture Decision Proposals (ADPs)

In Aipo engineering, we adhere to a strict **no-invention policy**: no arbitrary technical decisions are made silently in code.

Every architectural question or design evolution is formally debated and recorded as an **Architectural Decision Proposal** (ADP), documenting context, evaluated options, explicit rationale, downstream documentation impacts, and automated acceptance criteria.

---

## Approved Decision Log

| ADP | Title | Status | Decision Summary |
| :--- | :--- | :--- | :--- |
| **[ADP-001](/en/decisions/adp-001)** | Core Types and Bytes as Values | Approved | Treats `Bytes` as first-class VM values; tolerant slicing and clamped bounds. |
| **[ADP-002](/en/decisions/adp-002)** | Construction Hooks and Runtime Contracts | Approved | Formalizes `init()` and `invariant()` hooks with transactional rollback inside `attempt`. |
| **[ADP-003](/en/decisions/adp-003)** | Execution Budgets | Approved | Instruction step metering mechanism for untrusted script sandboxing. |
| **[ADP-004](/en/decisions/adp-004)** | Unicode Identifier Policy | Approved | Mandatory Unicode NFC normalization for all identifiers and string literals. |
| **[ADP-005](/en/decisions/adp-005)** | Parser Recursion Limits | Approved | Hard recursion depth limit in parser to eliminate stack overflow Denial-of-Service attacks. |
| **[ADP-006](/en/decisions/adp-006)** | Open Decisions for Waves 3 & 4 | Approved | Resolves async semantics, task combinators, and capability scoping boundaries. |
| **[ADP-007](/en/decisions/adp-007)** | Package Identity & Distribution | Approved | `namespace.package` coordinates, commit SHA-pinned GitHub dependencies, and offline cache. |
| **[ADP-008](/en/decisions/adp-008)** | Scoping Language Release v0.1.0 | Approved | Language-first, closed-scope strategy; defers game engines and web registries post-v1. |
| **[ADP-009](/en/decisions/adp-009)** | Synchronous Versioned C ABI | Approved | Stable C binary interface with opaque types for embedding Aipo into C/C++/Zig. |
| **[ADP-010](/en/decisions/adp-010)** | Thin Interoperability Proofs | Approved | Three automated thin proofs demonstrating seamless embedding in Rust, C, and JavaScript. |
| **[ADP-011](/en/decisions/adp-011)** | Performance & Ergonomics Roadmap | Approved | `Value` compaction, `InvokeMethod` instruction fusion, hoisted loop dispatch, and pattern matching. |
| **[ADP-012](/en/decisions/adp-012)** | Modern Syntax Ergonomics (Gleam/Swift Pivot) | Approved | Curly brace `{ ... }` blocks, immutability by default in structs, `var self`, symmetric `:`, and automatic structural interfaces. |
