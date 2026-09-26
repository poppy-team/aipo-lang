# Language Conformance Matrix

The Aipo **conformance suite** guarantees that language semantics and specifications are maintained across all builds and targets.

---

## Conformance Suite Architecture

Located under `docs/conformance/`, the suite is divided into three primary pillars:

### 1. Canonical Programs (`docs/conformance/programs/`)
Over 28 end-to-end integration programs:
- `01_hello.aipo`: Initialization, string literals, and standard output.
- `02_recursion.aipo`: Stack frame depth and scope isolation.
- `03_control_flow.aipo`: `if/else` conditionals, `while`, `loop`, and `repeat` cycles.
- `04_collections.aipo`: List and dictionary operations.
- `05_structs_and_impl.aipo`: Struct declarations and associated method blocks.
- `06_closures.aipo`: Lexical captures and shared mutable upvalues.
- `08_failures.aipo`: Operational fault handling via `fail`.
- `12_bytes.aipo`: Binary data packaging and byte manipulation.
- `13_init_and_invariant.aipo`: `init()` hooks and `invariant()` predicates.
- `15_invariant_on_mutation.aipo`: Invariant validation upon struct field mutations.
- `16_interface_contracts.aipo`: Runtime structural interface contract enforcement.
- `21_attempt_recovery_and_journal.aipo`: Transactional atomic state rollback.
- `24_async_functions_and_await.aipo`: `async fn` definitions and `await do` blocks.
- `26_async_combinators.aipo`: Async task combinators (`task.all`, `task.race`, etc.).
- `28_time_clock_capability.aipo`: Sandboxed `clock` capability verification in Host ABI.

### 2. Diagnostic Assertions (`docs/conformance/diagnostics/`)
Negative test cases verifying that invalid syntax or illegal operations emit precise diagnostics:
- `AIPO_LEX_INVALID_NUMBER`: Malformed decimal numbers lacking trailing digits.
- `AIPO_SEM_FIXED_REASSIGN`: Attempting to mutate an immutable `fixed` field.
- `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`: Using `await` outside a sequential block.
- `AIPO_SEM_FORGOTTEN_TASK`: Spawning an unawaited, unmanaged asynchronous task.
- `AIPO_RT_AWAIT_CYCLE`: Runtime task await cycle and deadlock detection.

### 3. Formatting Conformance (`docs/conformance/formatting/`)
Idempotency and layout test cases verifying the canonical formatter `aipo fmt`.
