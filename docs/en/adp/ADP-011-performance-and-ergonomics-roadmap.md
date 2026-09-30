# ADP-011 — Structural performance and language ergonomics roadmap

**Status:** accepted
**Date:** 2026-09-26
**Related:** `docs/performance/baseline.md`, `docs/performance/cross-language.md`, `docs/adp/ADP-008-v0.1.0-language-release.md`, `crates/aipo-vm/`, `crates/aipo-bytecode/`
**Authority:** subordinate to the canon; defines the medium- and long-term architectural strategy for overcoming performance bottlenecks and for the ergonomic evolution of the Aipo language.

## A. Context and motivation

During phase P04 and the start of P05, seven local micro-optimization experiments (name pool, dense layout with 2 vectors, per-slot fixed-flag cache, guarded-type cache, method-metadata cache) failed to produce reproducible gains and were reverted.

The in-depth architectural audit showed that the Aipo interpreter operates with a **structural cost floor of ~100ns per opcode (~300 CPU cycles)**. Local micro-optimizations cannot get past that floor. Comparative measurements against `Wren 0.4.0` and `Luau 0.739` revealed gaps of 7x to 22x in arithmetic computation and of 10x to 14x in field access and method dispatch.

In addition, on the ergonomics side, the language lacks structured typed failures (currently limited to plain-text messages), pattern matching with destructuring, and static type narrowing.

## B. Architectural performance decisions

### P1. Compacting `Value` to 16/24 bytes
- **Decision:** Remove inline-allocated structures from the variants of the `Value` enum. The `Value::Native` variant starts using `Rc<NativeData>` or a catalog index, and `Value::Function` uses compact integers (`u32` for `entry_ip`, `u16` for `arity`).
- **Impact:** The size of `Value` drops from 48 bytes to 16 to 24 bytes, reducing memory bandwidth pressure by up to 66% and quadrupling the density of values per L1 cache line.

### P2. Method invocation fusion (`InvokeMethod`)
- **Decision:** Compile direct method calls `expr.method(args...)` as a fused `InvokeMethod` opcode, instead of the `GetField` + `Call` sequence.
- **Impact:** Complete elimination of the 200,000 ephemeral `Rc<BoundMethodData>` allocations observed in the `fields` workload.

### P3. Dispatch loop with hoisted variables
- **Decision:** Integrate the execution loop into a single continuous function instead of invoking `self.step()` per instruction, promoting critical pointers (`ip`, `stack_base`, `code`) to CPU-register-allocated local variables.

### P4. Superinstructions and opcode specialization
- **Decision:** Introduce specialized variants `Call0..Call4` and `GetLocal0..GetLocal3` with no extra operand bytes, increasing bytecode density and reducing decode cycles.

## C. Language ergonomics decisions

### E1. Structured failures (Model B+)
- **Decision:** Allow `fail` to accept any value or struct instance as the failure payload, preserving automatic invariant rollback and extending `attempt ... failed err` to typed inspection (`if err is MyError`).

### E2. Pattern matching and destructuring
- **Decision:** Introduce the `match expr` expression with structural destructuring of structs, lists/tuples and primitive types.

### E3. Static type narrowing in `aipo-sema`
- **Decision:** Implement flow-sensitive refinement on `is` checks, eliding the emission of `AssertContract` instructions at runtime when the type is guaranteed at compile time.

## D. Validation and conformance protocol

All implementations arising from this ADP must:
1. Be validated with the paired harness `scripts/perf/paired.sh` under CPU-time methodology (`cpu_ab.py`) with multiple batches.
2. Maintain 100% differential parity between the VM and the JavaScript backend (`aipo-js`).
3. Preserve the complete conformance suite and the project's 5 quality gates.
