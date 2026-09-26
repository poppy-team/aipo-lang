# The Optimization Saga & Methodological Rigor

A central engineering takeaway from the Aipo project is the **discipline of empirical measurement**: optimizations that look compelling in theory frequently deliver zero speedup when subjected to controlled paired benchmarking.

Documenting experiments that were **reverted** is just as crucial as highlighting durable optimizations.

---

## Proven & Retained Speedups

Across VM and runtime profiling, the following optimizations demonstrated statistically verified gains:

1. **Monomorphic Field Slot Cache**:
   - Decreased `fields` benchmark execution time from 1,001.77ms down to 718.20ms (**28.31% reduction**).
2. **Base Frame Pointer Caching**:
   - Avoided redundant `frames.last()` queries during local slot lookups, producing consistent speedups: **-3.71% in `arithmetic`**, **-6.65% in `fields`**, and **-4.17% in `recursion`**.
3. **Eliminating Clone Calls in `SetField`**:
   - Deterministically eliminated 600,000 `Value` clones on unguarded struct mutations while preserving transactional rollback semantics.

---

## Seven Reverted Experiments

Seven optimization hypotheses were implemented, benchmarked using paired script `scripts/perf/paired.sh`, and **reverted** after evidence disproved measurable gains:

| Experiment | Technical Hypothesis | Measured Outcome | Decision |
| :--- | :--- | :--- | :--- |
| **1. Method Metadata Cache** | Avoid repeated method table lookups | No statistically significant difference | **Reverted** |
| **2. Name Pooling (`Rc<str>` vs Enum)** | Reduce allocations in identifier resolution | Keying overhead exceeded memory savings | **Reverted** |
| **3. Guardedness in Cache Entry** | Elide redundant dynamic integrity checks | Neutral wall-clock impact | **Reverted** |
| **4. `fixed` Flags per Slot** | Compact bitmask for immutable struct fields | Noise within margin of error (±2%) | **Reverted** |
| **5. Guarded Types Cache** | Reuse invariant enforcement schemas | Zero gain due to L2 cache contention | **Reverted** |
| **6. Dense `StructInstance` Layout** | Contiguous flat buffer for primitive fields | Offset calculation drifted +7% to neutral | **Reverted** |
| **7. Zero-Alloc Method Dispatch** | Elide intermediate wrapper pointers | Compiler already hoisted fast-path allocations | **Reverted** |

---

## Dispatch Loop Diagnosis

A calibrated fixed-iteration benchmark revealed a fundamental characteristic:
- Opcode execution cost in the VM is **constant (~100ns)**, regardless of instruction mix.
- The structural bottleneck lies in the outer dispatch loop and the bulky 72-byte `VmFault` type.
- Attempting to rewrite opcode decoding without `Result` yielded **zero speedup (-0.16%)**, confirming that LLVM was already sinking fault construction into cold branches off the hot execution path.

---

## Canonical Rules of Measurement

To prevent wasted effort under noisy CPU scheduling and background system contention:

1. **Any delta under ±5% is not evidence of speedup**.
2. **A single batch of measurements proves nothing**: every benchmark candidate requires at least two interleaved runs with candidate/baseline order inversion.
3. **`arithmetic` is the mandatory control workload**, as it isolates pure loop overhead from structs and dynamic method dispatch.
4. **Under CPU contention, CPU process time (`RUSAGE_CHILDREN`) is the only reliable signal**. Wall-clock measurements in shared environments produce false positives.

The micro-optimization phase for the baseline interpreter is officially **closed**. Any future work requires dedicated benchmark hardware and macro-architectural changes (`Call0..Call4`, `GetLocal8`, or threaded dispatch jump tables).
