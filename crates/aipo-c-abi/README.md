# aipo-c-abi

`aipo-c-abi` provides the versioned, synchronous, single-threaded C Application Binary Interface (C ABI) and embedding library for the Aipo programming language (`ADP-008`, `ADP-009`, `ADP-010`).

## Architecture & Guarantees

- **Single-Threaded Synchronous C ABI**: The execution context is strictly single-threaded, eliminating multi-threading races and lock overhead. No thread-safety guarantees are promised; host embedders orchestrate external synchronization if runtimes are accessed across multiple threads.
- **Fail-Safe Unwinding Boundary**: Every exported C ABI entrypoint intercepts panics using `std::panic::catch_unwind(AssertUnwindSafe(...))`. No Rust panic or lifetime escapes across the C FFI boundary, preventing undefined behavior or aborts.
- **Tagged Value Union (`aipo_value_t`)**: Plain C struct representing Aipo runtime values (`None`, `Bool`, `Int`, `Float`, `String`, `Bytes`, `Handle`, `Failure`). Integers and floats enforce the language's canonical invariants (+/- 2^53 - 1, finite-only).
- **Generational Handle Validation (`aipo_handle_t`)**: Opaque references to host objects validated via slot index and generation counters. Once released, handles reliably report `AIPO_ERR_STALE_HANDLE` rather than corrupting memory.
- **Capability Gating**: Host functions registered via `aipo_runtime_register_host_fn` can demand host capabilities (e.g. `system.vault`, `clock`, `io`). Unauthorized calls immediately fault with `AIPO_ERR_CAPABILITY_DENIED`.
- **String Memory Pooling**: Strings and raw bytes returned across the boundary are pinned in the runtime's memory pool and remain valid until the next runtime operation or until runtime destruction.
- **Diagnostic Error Reporting**: Comprehensive error diagnostics and runtime fault reasons are retrievable via `aipo_last_error`.

## Exported Header

The canonical header file is located at `include/aipo.h` and is ready for consumption by C, C++, Python ctypes, Go cgo, and other foreign function interfaces.

## Testing

```bash
cargo test -p aipo-c-abi
```

The test suite in `tests/c_abi_tests.rs` validates:
1. Version querying (`aipo_version`)
2. Runtime lifecycle allocation and cleanup without leaks
3. Module compilation and function calling with primitives (Int, Float, Bool, String)
4. Host native C callbacks called directly from Aipo scripts
5. Generational handle lifecycle, release, and stale detection
6. Capability guarding, granting, and revoking
7. Diagnostic compilation errors and runtime faults (e.g. division by zero)
8. Null pointer safety across all entrypoints
