# aipo-c-abi

`aipo-c-abi` provides the versioned, synchronous, single-threaded C Application Binary Interface (C ABI) and embedding library for the Aipo programming language (`ADP-008`, `ADP-009`, `ADP-010`).

## Architecture & Guarantees

- **Single-Threaded Synchronous C ABI**: The execution context is strictly single-threaded, eliminating multi-threading races and lock overhead. No thread-safety guarantees are promised; host embedders orchestrate external synchronization if runtimes are accessed across multiple threads.
- **Fail-Safe Unwinding Boundary**: Every exported C ABI entrypoint intercepts panics using `std::panic::catch_unwind(AssertUnwindSafe(...))`. No Rust panic or lifetime escapes across the C FFI boundary, preventing undefined behavior or aborts.
- **Tagged Value Union (`aipo_value_t`)**: Plain C struct representing Aipo runtime values (`None`, `Bool`, `Int`, `Float`, `String`, `Bytes`, `Handle`, `Failure`). Integers and floats enforce the language's canonical invariants (+/- 2^53 - 1, finite-only).
- **Generational Handle Validation (`aipo_handle_t`)**: Opaque references to host objects validated via slot index and generation counters. Once released, handles reliably report `AIPO_ERR_STALE_HANDLE` rather than corrupting memory.
- **Capability Gating**: Host functions registered via `aipo_runtime_register_host_fn` can demand host capabilities (e.g. `system.vault`, `clock`, `io`). Unauthorized calls immediately fault with `AIPO_ERR_CAPABILITY_DENIED`.
- **Runtime-Owned Snapshots (`aipo_value_release`)**: Heap payloads (`String`, `Bytes`, `Failure`) are returned as runtime-owned snapshots with trailing sentinel bytes. They are released explicitly by the host via `aipo_value_release` or upon runtime destruction, eliminating use-after-free and dangling pointer risks.
- **Reentrancy and Aliasing Guards**: Calling back into the runtime (`aipo_runtime_call`, `load_module`, `grant_capability`, `revoke_capability`, `register_host_fn`) or destroying it from within a host callback is intercepted and refused with `AIPO_ERR_USAGE`, preventing mutable aliasing violations on active VM frames.
- **Checked Handle Minting (`aipo_handle_create_checked`)**: Rejects out-of-contract values (integers > ±(2^53 - 1), NaN/infinite floats, non-NFC strings, or invalid UTF-8) with `AIPO_ERR_USAGE` without creating erroneous handles to `none`.
- **Module Isolation & Zero-Clone Invocations (M1 / P1)**: Functions are resolved strictly within module scope using shared `Arc<BytecodeModule>` references, eliminating cross-module function shadowing and per-call bytecode cloning.
- **Atomic Two-Stage Module Loading (M2)**: Definition tables and globals are snapshotted prior to initialization; top-level faults trigger transactional rollbacks, preventing VM state pollution.
- **Execution Budgets & Instruction Metering (S1)**: Runtime-enforced instruction quotas (`aipo_runtime_set_instruction_budget`) reliably terminate infinite loops with `AIPO_ERR_FAULT`, with queryable and resettable counters.
- **Diagnostic Error Reporting**: Comprehensive error diagnostics and runtime fault reasons are retrievable via `aipo_last_error`.

## Exported Header

The canonical header file is located at `include/aipo.h` and is ready for consumption by C, C++, Python ctypes, Go cgo, and other foreign function interfaces. It defines ABI version `0.2.0`.

## Testing

```bash
cargo test -p aipo-c-abi
```

The test suites validate:
1. `tests/c_abi_tests.rs`: Version querying, runtime lifecycle, module execution, host callbacks, generational handles, capability guarding, diagnostics, and null pointer safety.
2. `tests/c_abi_p0_tests.rs`: Dedicated regression tests pinning down the 7 boundary defects (C1–C7) identified in the technical dossier.
3. `tests/c_abi_p1_tests.rs`: Reproduction and regression tests for module isolation (M1), atomic rollback (M2), and execution budgets (S1).
4. `tests/host/main.c` + `tests/c_host_tests.rs`: Pure C executable compiled with native C compilers (`clang`/`gcc`) against `include/aipo.h`, linked with `libaipo_c_abi`, and executed under AddressSanitizer and UndefinedBehaviorSanitizer (`-fsanitize=address,undefined`).
