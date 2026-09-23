# Evidence — P03-G01 / Host ABI: Capability Model, Host Values and Generational Handles

**Goal:** `P03-G01` — Prove the host-neutral ABI defined by Wave 4: consume an AHS (host surface description as data), a deny-by-default capability model, host values that copy by value with host-owned external identity, and generational handles that never use-after-free (stale access yields none or a Failure per contract).
**Phase:** P03 (Wave 4 — Host ABI + Poppy) · **Recorded:** 2026-09-21
**Environment:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (all executed, all green)

```
$ cargo fmt --all -- --check                                           # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings                # 0 warnings
$ cargo test --workspace --all-targets                                 # 377 passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 warnings
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-cli --test conformance                            # programs 1–28, diagnostics 1–29
$ cargo test -p aipo-vm --test host_scope_escape                       # 9 passed (6 opcode sites + scope boundary)
$ cargo test -p aipo-vm --test host_capability_and_handles             # 5 passed (capability + stale handle)
$ cargo test -p aipo-host                                              # 37 unit + 1 doctest
```

## Acceptance Criteria Verification

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | Capability model is deny-by-default with the canon hierarchy | ✅ | `CANON_CAPABILITIES` in `aipo-host/src/capability.rs`; `CapabilitySet::none()` starts empty; 8 tests in `capability.rs` cover subsumption, narrowing, denial fault, and canon hierarchy well-formedness |
| 2 | AHS is consumed as data | ✅ | `HostSchema` in `aipo-host/src/ahs.rs` with `from_json`, `validate`, `declared_capabilities`, `missing_capabilities`, `require_capabilities`, `function` lookup; 9 tests cover well-formed surfaces, duplicates, subjects, async, malformed caps, empty host, invalid JSON |
| 3 | Host values copy by value; no Rust reference crosses the boundary | ✅ | `HostValue` in `aipo-host/src/value.rs` is a closed enum of plain data + `Handle`; `host_value_to_value`/`value_to_host_value` in `aipo-vm/src/host.rs` copy each variant by value with NFC normalization at the boundary; richer values (collections, functions) return `None` from `value_to_host_value`; 6 unit tests |
| 4 | Generational handles never use freed state | ✅ | `HandleTable` in `aipo-host/src/handle.rs` bumps generation on `remove`; exhaustion retires the slot instead of wrapping; `resolve` via `HostContext` returns `VmFault::StaleHandle`; 10 unit tests + 2 VM pipeline integration tests (`test_stale_handle_via_host_context`, `test_stale_handle_across_slot_reuse`) |
| 5 | Missing capability → AIPO_RT_CAPABILITY_DENIED with target/capability context | ✅ | `CapabilitySet::require` → `HostFault::CapabilityDenied` → `VmFault::CapabilityDenied`; pipeline test `test_capability_denied` with `revoke_clock()` proves the fault fires at runtime; `time.now`/`time.monotonic` test the positive path with `install_clock` |
| 6 | Scoped binding cannot escape, enforced at 6 heap-publication points | ✅ | `Vm::publish_check` called at `SetGlobal` (L132), `Return` (L263), `SetField` (L381), `SetIndex` (L546), `BuildList` (L604), `BuildDict` (L619-620) in `dispatch.rs`; `HostContext::ensure_publishable` walks containers recursively with visited set; 8 pipeline-level tests in `host_scope_escape.rs` covering all 6 sites |
| 7 | Nondeterministic sources through capability layer, replaceable by test profile | ✅ | `time.now`/`time.monotonic` gated by `ClockSource` trait; `install_clock`/`revoke_clock` process-global; CLI installs `SystemClock`; tests install `FixedClock`; denial faults with `AIPO_RT_CAPABILITY_DENIED`; conformance program 28 |
| 8 | fmt, clippy, test, doc all green | ✅ | See gates above |

## Summary of Implementation

| Area | Details |
|---|---|
| **`aipo-host` crate** | `#![forbid(unsafe_code)]`. Five modules: `ahs` (AHS as data), `capability` (deny-by-default hierarchy), `fault` (5-variant `HostFault` → stable codes), `handle` (generational `HandleTable`), `value` (`HostValue` closed enum with boundary checks). Depends only on `aipo-diagnostics` and `serde`. 37 unit tests + 1 doctest. |
| **VM adapter (`aipo-vm/src/host.rs`)** | `HostContext` holds `CapabilitySet`, `HandleTable<HostValue>`, open scopes, and escaped handles. Single-point conversions: `host_value_to_value` (NFC, range checks), `value_to_host_value` (plain data only), `host_fault_to_vm_fault` (total mapping, never a panic or recoverable `Failure`). 14 unit tests covering round-trips, out-of-range faults, NFC normalization, reference refusal, capability denial, stale handles, scoped escape through containers and structs, self-referential walk termination, and full fault mapping. |
| **Scope-escape enforcement** | `publish_check` at 6 bytecode publication points guards values about to become heap-reachable. Early-exit on `!has_escapes()` (one `bool` test) means programs without host bindings pay effectively nothing. The walk handles `Value::List`, `Value::Set`, `Value::Dict`, `Value::Struct` with a visited-set to stop at self-referential values. |
| **`time` module** | `time.now()` → wall clock as `Duration`, `time.monotonic()` → monotonic clock as `Duration`. `ClockSource` trait with `install_clock`/`revoke_clock` for process-global swap. `SystemClock` uses `std::time::{SystemTime, Instant}`. Denial is a fault, never silent absence. The JS backend mirrors the same gate via `globalThis.__aipoClock`. |
| **Diagnostic codes** | `AIPO_RT_CAPABILITY_DENIED`, `AIPO_RT_STALE_HANDLE`, `AIPO_RT_SCOPE_ESCAPE` in `DiagnosticCode` with `Severity::Fault`. Documented in `docs/diagnostics/catalog.md`. |

## Test Inventory & Verification

Unit and integration tests:

- `crates/aipo-host/src/` — 37 unit tests + 1 doctest: capability subsumption and narrowing, deny-by-default, handle insert/get/release/stale/generation-exhaustion, value range checking, AHS validation.
- `crates/aipo-vm/src/host.rs` — 14 unit tests: value round-trips, out-of-range faults, NFC normalization, reference refusal, capability denial and grant, stale handles, scope escape through containers and structs, self-referential walk, fault mapping.
- `crates/aipo-vm/tests/host_scope_escape.rs` — 9 integration tests driving the real pipeline: SetGlobal, Return, BuildList, BuildDict, SetIndex, SetField escape sites; open-scope passthrough; no-host-bindings fast path; slot reuse after scope close.
- `crates/aipo-vm/tests/host_capability_and_handles.rs` — 5 integration tests: capability denied (revoke_clock → time.now/monotonic faults), capability granted (install_clock → success), stale handle via release, stale handle across slot reuse, live handle resolution.
- `crates/aipo-stdlib/src/time.rs` — 6 unit tests: denied/granted clock, fixed/ticking sources, wall reading.

Conformance corpus:

- `docs/conformance/programs/28_time_clock_capability.aipo` — monotonic ≥ 0, wall > 0, monotonic non-decreasing.

## Non-Goals (Explicit Deferrals)

| Item | Rationale |
|---|---|
| ECS scopes, command buffer, behaviors/events, game.random | `aipo-poppy` (P03-G02) |
| Any specific engine inside `aipo-host` | Stays general abstractions only |
| filesystem/network/process stdlib modules | Wave 6 capability stdlib |
| Instruction/fuel, heap and wall-time budget enforcement | ADP-003 (undecided) |
