# Cross-Language Benchmarks

Aipo provides a standardized benchmarking harness (`aipo-bench --compare`) evaluating wall-clock execution speed and memory footprint (*Peak RSS*) against established runtime ecosystems.

---

## Benchmark Scope

Our goal is measuring real-world workload execution costs without obscuring architectural distinctions between interpreters, JIT compilers, and ahead-of-time compiled native code:

- **Aipo VM In-Process**: Compilation occurs before the clock starts; measures VM instantiation, call frame allocation, and bytecode instruction dispatch.
- **Aipo CLI / VM (`aipo run`)**: Cold process startup (source reading, lexing, parsing, semantic analysis, bytecode emission, and execution).
- **Aipo → JavaScript (Node.js)**: Emitted JavaScript executed on the V8 engine via `aipo-js`.
- **Reference Runtimes**: Lua 5.4, LuaJIT, Wren 0.4.0, Luau 0.739, CPython 3.12, PyPy 8.0.0, Ruby, and native Rust (algorithmic baseline).

---

## Six Canonical Workloads

All benchmarks compute a standardized `checksum:<value>` to verify that optimizing compilers did not elide legitimate computation:

| Workload | Measured Domain | Primary Evaluation Goal |
| :--- | :--- | :--- |
| `arithmetic` | Tight loop with function calls and intensive math | Basic dispatch overhead and call frame mechanics |
| `collections` | Dynamic list construction and traversal | Heap allocation pressure and container indexing |
| `fields` | Struct with 6 fields, mutation, and method calls | Property lookups and monomorphic field caching |
| `strings` | Incremental ASCII string concatenations | Text buffer growth and reallocation characteristics |
| `recursion` | Deep recursive calculation of Fibonacci(24) | Call stack depth and pure invocation overhead |
| `startup` | Minimal process initialization and exit | Cold startup time and standard library bootstrap |

---

## Reproducing the Results

To reproduce these measurements locally:

```bash
# Compile binaries in optimized release mode
cargo build --release -p aipo-cli -p aipo-bench

# Execute the complete comparison suite
target/release/aipo-bench --compare --compare-json target/cross-language.json

# Quick developer smoke run
target/release/aipo-bench --compare --compare-quick
```
