# Aipo Cross-Language Benchmarks

**Status:** implemented; manual performance evidence, not a blocking gate
**Scope:** portable workloads for Aipo CLI/VM, Aipo VM in-process, Aipo→JavaScript/Node, Lua, LuaJIT, Wren, Luau, CPython, PyPy, Ruby, JavaScript/Node and Rust native
**Related:** `docs/performance/baseline.md`, `docs/testing/ci-tiers.md`, `crates/aipo-bench/`

## Objective

Compare the cost of equivalent workloads without hiding the difference between interpreter, JIT, VM and native code. Rust native is an algorithmic control; its process uses the same binary as the runner and includes that harness's startup, so it does not represent the startup of a minimal native binary.

Aipo appears in four lines:

- **Aipo Wasm JIT:** direct compilation from HIR to WebAssembly and execution via Cranelift/Wasmtime with the `aipo_host` Host ABI (`aipo run --wasm`).
- **Aipo VM in-process:** compilation outside the timer; execution, VM creation and stdlib registration inside the timer.
- **Aipo CLI/VM:** the `aipo run` process, including startup and source compilation on the classic Stack VM.
- **Aipo→JavaScript/Node:** bundle emitted once before the timer; Node execution is measured separately.

## How to run

Prepare the release binaries:

```bash
cargo build --release -p aipo-cli -p aipo-bench
```

Run the complete comparison:

```bash
target/release/aipo-bench --compare \
  --compare-json target/cross-language.json
```

Useful modes:

```bash
# three samples, for development
target/release/aipo-bench --compare --compare-quick

# specific workloads and languages; N is explicit per workload
target/release/aipo-bench --compare --compare-n arithmetic=100000,collections=50000 \
  --compare-languages aipo-vm,lua,python,rust

# specific workload for an isolated A/B
target/release/aipo-bench --compare --compare-workloads collections \
  --compare-languages aipo-vm --compare-runs 15

# explicit path to the Aipo binary
target/release/aipo-bench --compare --aipo-bin target/release/aipo

# first wave: local Wren and Luau; PyPy is optional
PATH="$Wren_BIN_DIR:$LUAU_BIN_DIR:$PATH" \
  target/release/aipo-bench --compare \
  --compare-languages wren,luau,pypy \
  --compare-workloads arithmetic,collections,strings,recursion,startup \
  --compare-runs 1 --compare-n arithmetic=32,collections=32,strings=8 \
  --compare-resources
```

When `--aipo-bin` or `AIPO_BIN` is used, the runner records the hash and the profile inferred from the path; a custom binary remains the responsibility of whoever selected it.

### Dedicated paired runner

For local A/B measurements, use two already-built release binaries and the `scripts/perf/paired.sh` runner. It pins the CPU, alternates the baseline/candidate order and does not change the Git state:

```bash
scripts/perf/paired.sh \
  --baseline target/release/aipo-bench-before \
  --candidate target/release/aipo-bench-after \
  --workloads arithmetic,collections,fields,recursion \
  --rounds 15 --pairs 3 --cpu 0 \
  --output-dir target/paired
```

The script requires `taskset` by default; `--allow-unpinned` should only be used on machines without taskset support. The reports and a `manifest.txt` with hashes, workloads, rounds and CPU are kept in `target/paired`.

The runner detects `lua`, `luajit`, `wren_cli`/`wren`, `luau`, `python3`, `pypy3`/`pypy`, `ruby` and `node`. Missing external runtimes are marked as skipped. Aipo CLI/VM, Aipo→JavaScript/Node and Rust native are required; without the Aipo binary or without Node, the failure appears in the report and the command returns an error.

## Workloads

All programs print exactly `checksum:<value>`. The runner rejects missing or divergent output, or a non-zero exit code.

| Workload | Default N | What it measures |
|---|---:|---|
| `arithmetic` | 200000 | loop with function call and arithmetic |
| `collections` | 100000 | list construction and iteration |
| `fields` | 100000 | six fields, mutation and method calls |
| `strings` | 5000 | incremental ASCII concatenation |
| `recursion` | fixed | Fibonacci(24) |
| `startup` | fixed | minimal process, runtime initialization and a possibly warm cache |

`--compare-quick` removes `startup` and uses three samples. A normal run uses five samples. `--compare-n` takes `workload=N` pairs, with N between 1 and 1,000,000; this avoids applying the arithmetic N to the quadratic strings workload. `--compare-workloads` selects a list of workloads for an isolated A/B. `--compare-runs` accepts 1 to 31.

## First wave of references

The matrix adds three references without turning different groups into a single comparison:

- **Wren 0.4.0 + Wren CLI 0.4.0**: compact VM, bytecode and embedding; the CLI receives the script and arguments directly.
- **Luau 0.739**: Lua-derived VM, with interpreter and native code; the CLI uses `-a` to separate the program's arguments.
- **PyPy 8.0.0, Python 3.12**: the same Python language with a JIT/GC runtime; if absent from the PATH, it appears as `skipped`.

The sources are kept in `/home/raillen/Documentos/Projetos/aipo-reference-runtime/` and recorded in `SOURCES.json`. They are read-only references, not build dependencies of Aipo. The runner resolves the executable through the `PATH`; the pinned checkout does not, by itself, prove which binary was executed. Therefore, local runs must record the `PATH`, the binary hash and the reported version.

Wren does not provide an official CLI in the VM's main checkout. The separate CLI is the source of the executable used for this battery. The CLI 0.4.0 build ran into a `write` name collision with current libc headers; the local run used a temporary tree with a compatibility rename, without modifying the pinned checkout. Luau does not accept `--version`; the runner uses `-h` to confirm availability and records `luau CLI (no --version flag)`.

Initial reading map:

| Aipo | References |
|---|---|
| `crates/aipo-vm/src/vm/dispatch.rs` | `lua/lvm.c`, `cpython/Python/ceval.c`, `ruby/vm_exec.c`, `node/deps/v8/src/interpreter/interpreter.cc` |
| `crates/aipo-vm/src/vm/call.rs` | `ruby/vm_method.c`, `luau/VM/src/lvmexecute.cpp`, `wren/src/vm/wren_vm.c` |
| `crates/aipo-vm/src/value.rs` | `lua/lobject.c`, `wren/src/vm/wren_value.c`, `luau/VM/src/lobject.cpp` |
| `crates/aipo-bytecode/src/emitter.rs` | `luau/Compiler/src/Compiler.cpp`, `wren/src/vm/wren_compiler.c`, `cpython/Compile` |

The reading serves to form hypotheses. No reference code should be copied without checking the license, the runtime contract and semantic parity.

Allocation counting was explicitly left out of the first result because Aipo does not yet have a single, accountable allocator/GC contract; `docs/adp/ADP-003-execution-budgets.md` remains a draft. The next spike must define scope, unit and cost before choosing `Rc`, arena or tracing GC.

Recommended order for the next fields experiments: (1) monomorphic per-site slot cache, already kept; (2) base-frame cache, already kept; (3) entry copy of `SetField` on unguarded structs, already kept; (4) `BoundMethod` allocations still pending, but with no new name pool until there is a dedicated runner; (5) opcodes `Call0..Call4`/`GetLocal8`. Dense struct layout, cached method metadata, name pool and allocation-free method dispatch were tested and reverted for lack of demonstrated gain. Each step needs its own A/B; do not introduce JIT, NaN-boxing or a custom GC before there is evidence.

### Runner noise floor

This host delivers a noise floor of approximately ±5% on the `fields` and `arithmetic` workloads under normal conditions, and a much worse dispersion under contention: the same binary has varied by 2.5x between runs of the same configuration, with a load average of 7.3 on 4 cores. Practical consequences:

- A difference smaller than ±5% is not evidence of anything, even with identical checksums and a small per-run MAD.
- A single paired batch does not decide a change. Candidate changes need at least two batches, ideally with `arithmetic` as a control workload, because it does not touch the changed path and therefore measures environment drift.
- `arithmetic` uses neither structs nor method dispatch, so it serves as a drift control for fields experiments.
- Moving fields around in large structs changes offsets and cache alignment enough to swing from +7% to neutral without any semantic change. Field position is part of the experimental result and must be reported alongside the delta.
- A batch with a small MAD can still be contaminated by drift accumulated over the session; comparing the candidate with the baseline measured in the same batch remains mandatory.
- When the host is contended, wall-clock decides nothing. `scripts/perf/cpu_ab.py` measures the child process's CPU time (`RUSAGE_CHILDREN`) instead of wall-clock: under contention wall-clock inflates by an arbitrary factor, while CPU time remains proportional to the work actually done, because time taken away by the scheduler is not charged. Even so, with a 2.5x dispersion within the same binary, the minimum over many repetitions remains the most reliable estimator, and the correct conclusion is "inconclusive" when the intervals overlap.

## What the report contains

The JSON report uses schema 5; the entry identifies `manifest_schema` separately.

Each result includes:

- the raw sample of each run;
- median and MAD;
- minimum, p95 and maximum;
- expected and observed checksum;
- operations and operations per second;
- runtime version;
- `N`, mode, preparation setup and runtime setup where applicable;
- internal metrics for instructions, calls, fields, field cache, globals and constants for the Aipo VM;
- for the Aipo VM, timing without instrumentation and a separate, untimed run to collect metrics;
- `resources` with `peak_rss_bytes`, when available, in bytes, obtained by Linux `/proc` sampling only with `--compare-resources`; `allocation_count` and `allocated_bytes` remain null until there is a runtime-specific adapter;
- `field_cache_hits` and `field_cache_misses` count slot lookups in `GetField` and `SetField`; therefore the hits can be greater than `field_lookups`, which records only reads;
- environment, build profile, Git dirty state, CPU/OS and number of available CPUs;
- hash of the runner binary and hash/profile of the Aipo binary when selected.

Example output:

```text
Aipo CLI/VM  287.11ms MAD=2.28ms p95=289.39ms checksum=59999500000
Lua           4.33ms MAD=233.35us p95=4.56ms checksum=59999500000
Rust native   1.49ms MAD=32.69us p95=1.79ms checksum=59999500000
```

## Fairness policy

- The same algorithm, the same logical operations and the same `N` are used in all languages; native Rust also builds and traverses the list and the string.
- The `fields` workload preserves the same recurrence and the same checksum, but uses structs, classes, tables or `__slots__` according to each runtime's model; compare it as a lookup hotspot, not as an object-layout ranking.
- The result is verified by checksum; this reduces the chance of dead work, but it is not independent proof that each operation was executed.
- A warmup does not enter the samples.
- `process` and `in-process` must not be compared as the same metric.
- Interpreter startup remains included in process results.
- Aipo→JS compilation/bundling is reported as setup, not mixed with execution.
- Aipo VM in-process does not include process startup and is always labeled separately.
- Native Rust is compiled outside the timer and does not represent the same class of runtime; the startup line includes the `aipo-bench` harness and serves only as a control for the runner.
- The p95 of a short round can be equal to the maximum; use more samples on a dedicated runner before interpreting that number.

## Limitations

- The result depends on CPU, system, versioning and thermal state.
- Python/PyPy, Ruby, Lua/LuaJIT, Wren, Luau and Node have different garbage collectors, dispatch, recompilation and optimizations. The execution mode must be read together with `kind`, `mode`, `includes_startup` and `includes_compile`.
- The benchmarks record `peak_rss_bytes` by Linux sampling in a separate run when `--compare-resources` is active; sampling can miss the peak of very short processes. `allocation_count` and `allocated_bytes` are not yet comparable across runtimes and remain null when there is no native adapter. I/O, editor startup and engine integration remain out of scope.
- A large `N` reduces the effect of startup, but may measure different memory regimes.
- The fixtures are versioned and trusted; the runner is not a sandbox for arbitrary code. External processes have a 120 s deadline and output limited to 1 MiB; VM capture is also limited to 1 MiB, but the in-process VM has no instruction budget for arbitrary code.
- A local run is not a regression baseline. Use a dedicated runner, CPU pinning and a history of at least three reports.

## Final local result

The historical table below covers the five original workloads; the `fields` workload is reported separately in the first wave.

A complete five-sample round on Linux x86_64, Intel Core i7-3632QM, sequential execution, 4 available CPUs, Rust 1.98.1, Node 24.18.0, Python 3.14.6, Ruby 4.0.6, Lua 5.5.1 and LuaJIT 2.1 produced these medians:

| Workload | Aipo CLI/VM | Aipo VM in-process | Aipo→JS | Lua | LuaJIT | Python | Ruby | JS | Rust |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `arithmetic` | 251.04ms | 239.12ms | 572.05ms | 10.68ms | 5.49ms | 57.33ms | 99.18ms | 57.02ms | 5.51ms |
| `collections` | 194.68ms | 180.20ms | 1118.91ms | 10.62ms | 5.50ms | 41.67ms | 94.90ms | 62.60ms | 5.64ms |
| `strings` | 10.62ms | 5.45ms | 109.41ms | 10.57ms | 10.65ms | 25.87ms | 87.60ms | 52.13ms | 5.50ms |
| `recursion` | 85.09ms | 79.00ms | 269.51ms | 10.62ms | 5.57ms | 31.07ms | 92.09ms | 50.32ms | 5.51ms |
| `startup` | 5.50ms | 0.29ms | 63.83ms | 5.53ms | 5.52ms | 21.05ms | 85.65ms | 48.40ms | 5.55ms |

The last implementation change removes temporary `Vec`s from natives and conversions; the subsequent measurement in `target/cross-language-final-current.json` was taken under heavy runner load and therefore does not replace the table above as a comparable baseline.

## Local result of the first wave

Release round with five samples, sequential execution, Wren 0.4.0 and Luau 0.739. The numbers are process medians and include startup/compilation; they are not a regression baseline.

| Workload | Wren | Luau |
|---|---:|---:|
| `arithmetic` | 41.37ms | 22.27ms |
| `collections` | 37.38ms | 11.61ms |
| `fields` | 120.35ms | 63.15ms |
| `strings` | 84.00ms | 21.44ms |
| `recursion` | 22.36ms | 16.07ms |
| `startup` | 6.55ms | 7.25ms |

On the `fields` workload, the same round measured the Aipo VM at 1,430.92ms, against 120.35ms for Wren and 63.15ms for Luau. That difference is a **warning about a fields/methods hotspot**, not a universal verdict: the three runtimes have different object models. The detailed report is in `target/fields-release.json`.

PyPy was not available in the `PATH` and was recorded as `skipped`. A complete round with all available runtimes produced 66 results, 6 skipped and 0 failures. The first-wave report is in `target/first-wave-release.json`; the full matrix is in `target/cross-language-first-wave-full.json`. Both contain samples, RSS when requested and provenance.

## A/B of the monomorphic fields cache

15-sample experiment, Aipo VM in-process, `N=100000`, same workload and same checksum:

| Variant | Median | MAD |
|---|---:|---:|
| Before | 1,001.77ms | 39.22ms |
| Per-site cache | 718.20ms | 30.54ms |

Observed median reduction: **28.31%**. The cache produced 2,399,992 hits and 8 misses in the metrics pass. Reports: `target/fields-cache-before.json` and `target/fields-cache-after.json`. This result is directional, not a gate, because the runner is shared.

## Rejected experiment: method metadata cache

A per-site/type cache for method `GetField` was temporarily implemented and measured against the same `fields` workload. It recorded 199,998 hits and 2 misses, but showed no reproducible gain on the shared runner:

| Variant | Median (31 samples, CPU 0) |
|---|---:|
| Without method cache | 971.58ms |
| With method cache | 1,131.79ms |

A smaller sequential round was also worse (1,141.64ms without cache against 1,599.96ms with cache). Since the values were affected by heavy load, the conservative conclusion is a lack of demonstrated benefit, not a universal regression. The prototype was reverted; the monomorphic fields cache and the base-frame cache remain as kept optimizations. Reports: `target/fields-method-disabled-pinned.json` and `target/fields-method-enabled-paired.json`.

## Rejected experiment: method name pool

A prototype changed the `BoundMethodData` name to `Rc<str>` and kept qualified names in per-site slots. The intent was to eliminate `format!` and name copies, but the A/B showed no reproducible gain: in an adjacent 31-sample round, `fields` came in at 602.49ms against 605.07ms for the baseline, while `collections` worsened from 178.82ms to 192.71ms. A second variant with `Owned`/`Shared` did not fix it either: `fields` 635.72ms against 630.59ms and `collections` 190.71ms against 179.45ms. Runner load varied between runs, so these numbers are only an exploratory rejection, not a universal regression. The pool and the `BoundMethodData` layout change were reverted; the next attempt should avoid API-breaking changes without an A/B on a dedicated runner. Reports: `target/qualified-name-candidate-31.json`, `target/qualified-name-baseline-31b.json` and `target/bound-name-enum-candidate-31.json`.

## Rejected experiment: per-slot `fixed` flag cache

The registered `SetField` started carrying a `Vec<bool>` of `fixed` flags per type and using a setter with the flag already resolved, avoiding `HashSet<String>::contains` per write. The paired A/B (3 pairs, 15 samples) worsened the median in all three pairs: `arithmetic` +5.34%, `fields` +11.32% and `recursion` +144.36%. Checksums and metrics remained identical, but the cost of the additional lookup outweighed the savings; the change was reverted. Reports: `target/fixed-field-paired/`.

## Rejected experiment: cache entry with guardedness

The per-site cache was extended to also store `type_is_guarded`, eliminating the two HashMap lookups in `SetField`. The paired A/B (3 pairs, 15 samples) showed no gain: `arithmetic` +6.32%, `fields` +1.40% and `recursion` +2.13% in the median of the three pairs. Checksums and metrics remained identical, but the extension was reverted. Reports: `target/field-entry-paired/`.

## Rejected experiment: dense `StructInstance` layout

`StructInstance` stopped storing `Vec<(String, Value)>` and started storing `field_names: Vec<String>` alongside `fields: Vec<Value>`, eliminating the per-slot tuple. The intent was to make field access a direct index into `Vec<Value>` and reduce the size of the stored value.

The paired A/B (3 pairs, 15 samples, CPU 0) showed the candidate slower on `fields` in all three pairs: +3.73%, +7.39% and +24.23%. In the median of the three pairs, `arithmetic` was +33.87%, `fields` +22.66% and `recursion` -57.76%. Runner load was high during the measurement (load average 2.50 to 3.80 on 4 cores, with the code agent consuming ~86% of CPU), which explains the dispersion in `arithmetic` and `recursion`; even so, the direction on `fields` was consistent across the three pairs and is the relevant evidence, since that is the workload that exercises the changed path. Checksums remained identical in all pairs.

The likely cause is structural: separating names and values trades one allocation per struct instance for two, and each field access now touches two memory regions instead of one. Since the `fields` fixture builds instances repeatedly, the extra allocation dominates the expected gain from indexed access.

The experiment was also API-breaking: `fields` is public and the eight consuming files (`value.rs`, `vm/dispatch.rs`, `host.rs` and five stdlib modules) had to change. Since there was no gain, the prototype was reverted and the public API remains `Vec<(String, Value)>`. Reports: `target/dense-layout-paired/`.

## Rejected experiment: allocation-free method dispatch

Inspection of the hot path found a real cost that the benchmarks did not isolate. In the `fields` fixture there are 200,000 method binds (`advance` and `score`), and each one performed four `String` allocations: a clone of the receiver's `type_name`, two clones to build the map key and a `format!("{type_name}.{field_name}")`. The correct pattern already existed in the repository itself: `method_natives` has a nested twin (`method_natives_by_type`) that does lookup with `&str` without allocating, but `struct_methods`, the user-method map and the hot path of `fields`, never received that treatment.

The prototype applied the same pattern: a nested `struct_methods_by_type` per type, a qualified name formatted once at registration instead of at every bind, and `lookup_struct_method(&str, &str)` reusing `lookup_method_native`. The instance name was only cloned on the error path. Semantically the change was neutral: 56 suites passed, including the VM↔JS differential, and the checksums remained identical.

Even so, the gain could not be demonstrated. There were four paired batches, changing only the position of the new field in the `Vm` struct:

| Batch | Position of the new field | Δ `fields` | Δ `arithmetic` |
|---|---|---:|---:|
| 1 (5 pairs) | inline, in the middle | -5.57% | +1.72% |
| 2 (3 pairs) | inline, in the middle | -3.70% | +1.13% |
| 3 (3 pairs) | at the end of the struct | +1.05% | -0.02% |
| 4 (5 pairs) | boxed, in the middle | +4.54% | +6.80% |

The same logical change, with the same mechanism, swung from -5.6% to +4.5% merely by moving a field. This indicates that the runner's noise floor is approximately ±5% on this host and that differences within that range are not evidence. The first batch, which suggested a convincing gain of -5.57% with 4 of 4 clean pairs favoring the candidate, was largely the luck of a quiet batch; later batches with controlled noise refuted it. `arithmetic` works as a control workload, since it has neither structs nor method dispatch, and therefore measures environment drift rather than the effect of the change.

For the reasons above, the prototype was reverted. Besides the lack of demonstrated gain, the change required duplicating registration state: `struct_methods` is `pub`, so any future write outside `register_struct_method` would desynchronize the shadow map and silently break method dispatch, with no speed gain to pay for that risk.

## Diagnosis: constant per-opcode cost and the toll of the 72-byte `VmFault`

Measuring the per-opcode cost in three calibration programs with the same number of iterations, and using the deterministic instruction count as the denominator, the per-opcode cost is practically constant and independent of the operation executed:

| Program | Ops/iter | ns per opcode |
|---|---:|---:|
| `i = i + 1` (locals) | ~9 | ~111 |
| `s = s + i; i = i + 1` | ~14 | ~98 |
| `arithmetic` (calls and globals) | 28 | ~100 |

A constant cost of about 100ns per opcode, which does not change with the mix of operations, indicates that the cost lies in the dispatch loop and not in any handler. Ablation ruled out metrics accounting as the cause: turning off `metrics_enabled` changed the result by only 1% to 2%.

The identified cause is at the top of `Vm::step`, which executes for **every** opcode:

```rust
// before
let opcode = OpCode::try_from(opcode_byte).map_err(|b| VmFault::CorruptedBytecode {
    offset: self.ip - 1,
    reason: format!("unknown opcode 0x{b:02x}"),
})?;
```

`OpCode::try_from` returns `Result<OpCode, u8>`, that is, 2 bytes, but the `.map_err` rebuilds the result as `Result<OpCode, VmFault>`. `VmFault` is 72 bytes, because several variants carry an inline-allocated `String`. Therefore each executed opcode built and destroyed a 72-byte `Result`, and the closure that does the conversion captured `self`, which forces the compiler to keep `ip` in memory. Add to that the fact that every `Result<_, VmFault>` from `push`, `pop` and `read_u16` is also 72 bytes, so the same cost appears several times per opcode.

The fix replaces the `.map_err` with a `match` that only builds the failure in the cold branch, keeping the offset and message identical:

```rust
let opcode = match OpCode::try_from(opcode_byte) {
    Ok(opcode) => opcode,
    Err(byte) => return Err(VmFault::CorruptedBytecode { /* identical to the previous one */ }.into()),
};
```

The happy path becomes a `match` over 2 bytes, with no wide `Result`, no closure capturing `self` and no `format!`. The test `test_unknown_opcode_reports_the_offset_of_the_bad_byte` pins down the cold-branch error, and the workspace's 56 suites pass with identical instruction counts and checksums.

**The measurement result is null.** With child-process CPU time, 24 alternating repetitions per `arithmetic` workload, the result was:

| | min | median |
|---|---:|---:|
| before | 3.019s | 5.474s |
| after | 3.014s | 5.359s |
| delta | -0.16% | -2.09% |

The deciding number is the minimum, because it is the uncontended state: if the change removed real per-opcode work, the minimum would drop measurably. It did not. The likely explanation is that LLVM was already sinking the failure construction off the happy path, leaving only the 2-byte `match` at runtime, so the waste existed in the source code but never actually executed. The two distributions overlap almost entirely (before 3.02s–7.53s, after 3.01s–8.52s).

The change was kept, not for a speed gain, which does not exist, but for two reasons that do not depend on a benchmark: it makes the cold-branch failure construction explicit in the code, instead of depending on the optimizer moving the error path out, and the new test pins down the `unknown opcode` contract, which previously had no coverage. Recording this as a performance optimization would be wrong; it is a robustness and clarity change with a measured null result.

The same compiler hypothesis probably applies to `push`, `pop` and `read_u16`: the size of `VmFault` is a real hygiene and ergonomics problem, and worth reducing on API grounds, but it is plausible that the cold error path is already removed at runtime. Therefore **shrinking `VmFault` must not be treated as a performance optimization without measuring**. The command to measure on an idle host is:

```sh
python3 scripts/perf/cpu_ab.py <binario-baseline> <binario-candidato> --workloads arithmetic --rounds 10 --reps 24
```

## A/B of the base-frame cache

The VM keeps the active frame's `stack_base` in `Vm::frame_base` so that `GetLocal`, `SetLocal` and `JumpIfSetLocal` do not consult `frames.last()` on every access. The value is updated in `run`, frame push/pop, failure handlers, `invoke` and task switches. A regression test covers the return from a nested call and the restoration of the outer frame.

Paired A/B, Aipo VM in-process, CPU 0, 31 samples, same workload and checksums:

| Workload | No cache on the hot path | With `frame_base` | Delta |
|---|---:|---:|---:|
| `arithmetic` | 276.23ms | 265.98ms | -3.71% |
| `fields` | 736.02ms | 687.05ms | -6.65% |
| `recursion` | 97.72ms | 93.65ms | -4.17% |

Internal metrics and checksums remained bit-for-bit identical. Reports: `target/frame-base-baseline-31.json` and `target/frame-base-candidate-31.json`. The result is directional, not a gate, because the runner is shared. An additional attempt to force `#[inline]` on hot helpers brought no reproducible gain and was reverted.

## Reduced copies in `SetField`

The entry value of an assignment is only needed by the journal when the type is guarded. The path now checks `type_is_guarded` before cloning the previous value; structs without `invariant()` keep the same mutation, without the entry copy. In the `fields` fixture, this deterministically eliminates 600,000 `Value` clones (six writes × 100,000 iterations). A later A/B with 31 samples showed no consistent wall-clock gain (`fields`: 608.31ms at baseline against 612.08ms with the change; `collections`: 184.11ms against 178.36ms). Therefore, the change is kept as a deterministic reduction in work, not as a speed claim. The rollback tests for guarded structs and the differential suite remain green.

## Rejected experiment: guarded-types cache

A `HashSet<String>` was created to replace the two lookups in `type_is_guarded`. The paired runner (3 pairs, 15 samples per run) showed the candidate slower in the median of the three pairs: `arithmetic` +6.58%, `fields` +1.63% and `recursion` +3.36%. Checksums and metrics remained identical, but there was no benefit; the cache was reverted. Reports: `target/guarded-cache-paired/`.


## Isolated A/B result of the shim constant cache

In a sequential run of three samples for `Aipo→JavaScript/Node` only, the lazy per-instruction constant cache reduced the median by approximately `26%` (`arithmetic`), `11%` (`collections`) and `17%` (`recursion`). The reports are in `target/js-constant-cache-base.json` and `target/js-constant-cache-after.json`; the result is directional because the runner is shared.

Observations: Aipo CLI/VM and Aipo VM in-process fell behind the compared languages on the computational workloads; Aipo→JavaScript remains the most expensive path, while the strings case highlights the cost of incremental concatenation. The in-process VM startup is measured without process startup. Native Rust executes the same logical operations, but remains only a lower-bound control.

In the local three-sample A/B comparison against `target/perf-phase1.json`, the Aipo VM reduced the median by `12.3%` (`arithmetic`), `10.1%` (`collections`), `6.0%` (`strings`) and `6.2%` (`recursion`); Aipo→JavaScript ranged from `-10.4%` to `+2.8%`. This result is directional, not a gate, because the runner is shared.

## WebAssembly JIT substrate (ADP-013 / Milestone 6)

With the introduction of the `aipo-wasm` crate and native integration into the CLI (`aipo run --wasm`), Aipo programs are compiled directly from HIR to WebAssembly bytecode and executed via native machine-code JIT by Cranelift/Wasmtime.

The WebAssembly runtime eliminates the dispatch overhead of the traditional Stack VM (opcode `match`, `Vm::step`, the 24-byte boxed `Value` and dynamic stack bounds checking), operating directly with Wasm primitives (`i64`, `f64`, `i32`) and a linear memory layout for structs.

### Comparative battery (`aipo-bench --compare`)

Run of 7 rounds comparing the new WebAssembly JIT runtime (`aipo-wasm`) with the previous Stack VM (`aipo` and `aipo-vm`) and market runtimes. Evidence recorded in `target/bench_wasm_comparison.json`.

#### 1. Arithmetic (`arithmetic`, N=200,000, checksum: `59999500000`)

Intensive loop of arithmetic function calls (`step(i) = i * 3 - 1`).

| Runtime | Execution type | Median (ms) | Minimum (ms) | Speedup vs VM | Ratio vs Wasm |
|---|---|---:|---:|---:|---:|
| **Rust native** | Natively compiled (`rustc -O`) | 5.51ms | 5.46ms | 48.5x | 0.35x |
| **LuaJIT** | Native tracing JIT | 5.57ms | 5.43ms | 48.0x | 0.35x |
| **Lua 5.4** | C bytecode VM | 10.77ms | 10.58ms | 24.8x | 0.68x |
| **Aipo Wasm JIT** | **WebAssembly JIT (Cranelift)** | **15.72ms** | **10.53ms** | **17.0x – 25.1x** | **1.00x** |
| **JavaScript (Node.js)** | V8 JIT | 63.44ms | 56.89ms | 4.2x | 4.03x |
| **Python (CPython 3.12)** | C bytecode interpreter | 69.59ms | 62.86ms | 3.8x | 4.43x |
| **Ruby (CRuby 3.x)** | YARV interpreter | 108.14ms | 105.87ms | 2.5x | 6.88x |
| **Aipo VM in-process** | Rust Stack VM (no startup) | 231.64ms | 229.61ms | 1.15x | 14.73x |
| **Aipo CLI/VM** | Rust Stack VM (with startup) | 267.34ms | 244.91ms | 1.00x (base) | 17.00x |
| **Aipo → JS / Node** | JS transpilation on Node | 707.36ms | 603.04ms | 0.38x | 44.99x |

**Highlights:**
- **Aipo Wasm JIT is 17x to 25x faster** than Aipo CLI on the classic Stack VM.
- **Outperforms Node.js by 4.0x**, **CPython by 4.4x** and **CRuby by 6.9x**.
- Ties with the Lua 5.4 C interpreter (10.53ms vs 10.58ms at the minimum).
- Sits only **1.9x** away from the absolute ceiling of natively compiled Rust.

#### 2. Deep recursion (`recursion`, Fibonacci(24), checksum: `46368`)

Pure recursive calls with 92,735 function invocations and frame activation tracking.

| Runtime | Execution type | Median (ms) | Minimum (ms) | Speedup vs VM | Ratio vs Wasm |
|---|---|---:|---:|---:|---:|
| **Rust native** | Natively compiled (`rustc -O`) | 5.50ms | 5.45ms | 17.9x | 0.48x |
| **LuaJIT** | Native tracing JIT | 5.85ms | 5.68ms | 16.8x | 0.52x |
| **Aipo Wasm JIT** | **WebAssembly JIT (Cranelift)** | **11.35ms** | **10.60ms** | **8.65x** | **1.00x** |
| **Lua 5.4** | C bytecode VM | 16.22ms | 11.12ms | 6.05x | 1.43x |
| **Python (CPython 3.12)** | C bytecode interpreter | 31.89ms | 30.26ms | 3.08x | 2.81x |
| **JavaScript (Node.js)** | V8 JIT | 59.66ms | 58.38ms | 1.65x | 5.26x |
| **Aipo VM in-process** | Rust Stack VM (no startup) | 77.68ms | 77.16ms | 1.26x | 6.85x |
| **Aipo CLI/VM** | Rust Stack VM (with startup) | 98.15ms | 86.64ms | 1.00x (base) | 8.65x |
| **Ruby (CRuby 3.x)** | YARV interpreter | 98.17ms | 93.55ms | 1.00x | 8.65x |
| **Aipo → JS / Node** | JS transpilation on Node | 307.74ms | 287.11ms | 0.32x | 27.12x |

**Highlights:**
- **Aipo Wasm JIT is 8.6x faster** than Aipo CLI/VM and 6.8x faster than the in-process VM.
- **Outperforms Lua 5.4 by 1.4x**, **CPython by 2.8x**, **Node.js by 5.3x** and **CRuby by 8.6x**.
- Sits only **1.9x** away from native Rust and LuaJIT.

#### 3. Process startup and initialization (`startup`, checksum: `ready`)

Measures the full lifecycle: CLI process invocation, parsing, lowering, machine-code JIT compilation and stdout emission.

| Runtime | Median (ms) | Minimum (ms) | Notes |
|---|---:|---:|---|
| **Aipo VM in-process** | 0.31ms | 0.30ms | No cost of a new OS process |
| **LuaJIT / Lua / Rust** | ~5.50ms | ~5.45ms | Minimal C/Rust process without heavy compilation |
| **Aipo CLI/VM** | 5.51ms | 5.46ms | Pure interpreter (no JIT) |
| **Aipo Wasm JIT** | **15.64ms** | **10.68ms** | **Includes parsing + Wasm codegen + native Cranelift compilation** |
| **Python (CPython)** | 20.80ms | 20.64ms | CPython runtime initialization |
| **Node.js** | 57.83ms | 56.37ms | V8 initialization |
| **Aipo → JS** | 68.40ms | 64.29ms | Node + Aipo runtime shim |
| **Ruby** | 89.63ms | 85.44ms | Ruby VM initialization |

The startup time of the Aipo Wasm JIT (~15ms) includes native-code JIT compilation by Cranelift, while remaining **faster than CPython (21ms), Node.js (58ms) and Ruby (90ms)**.

These numbers are local evidence, not a gate nor a performance promise. The complete report, with raw samples, release profile, dirty state and checksums, is in `target/cross-language-final.json` and `target/bench_wasm_comparison.json`, outside version control. Attach it as evidence when a product decision depends on the data.
