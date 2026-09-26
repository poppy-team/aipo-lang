# Wave 1 — Contracts, Rollback & Conformance

**Wave 1** elevated Aipo beyond conventional interpreters by introducing its defining architectural paradigm: **structural data guarantees through signature contracts, atomic rollback invariants, and formal interfaces**.

---

## Achieved Milestones

### 1. Construction Hooks and Invariants (`init` and `invariant`)
- Addition of the `init()` hook ensures that newly instantiated structures undergo validation and normalization before exposure to consumer code.
- The `invariant()` block was integrated into stable mutation boundaries. Whenever a struct field is mutated, the runtime rigorously validates declared logical assertions.

### 2. Transactional Rollback with `attempt ... recover`
- Implementation of an in-memory **mutation journal** inside the virtual machine:
  - Entering an `attempt` block initiates recording of all subsequent object mutations into a rollback journal.
  - If an operational failure (`fail`) occurs or an `invariant()` assertion is violated, the journal atomically reverts all modified structs to their pre-attempt state before transferring execution to the `recover` block.
  - Upon successful block completion, the journal is committed and discarded without lingering memory overhead.

### 3. Static and Runtime Interface Validation
- Support for `interface` declarations and `satisfy` assertions:
  - Semantic analysis pre-validates method names, parameter arities, and `self` receiver compatibility.
  - At runtime, calls dispatched through interfaces perform structural validation, yielding deterministic faults if an incompatible object is provided.

### 4. Full Support for Local Functions & Module Scopes
- Lexical scope resolution completed with nested local functions and self-referential recursion support via the `FillSelfCapture` bytecode instruction.
- Transparent and safe access to module-level bindings from `impl` methods and closures.

### 5. 100% Green Conformance Gauntlet
- Certified by 20 canonical programs and 19 diagnostic test suites, achieving 100% compliance across all quality gates (fmt, clippy, check, test, doc).
