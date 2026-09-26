# Aipo Language Manual

This manual formally documents syntax, data types, module boundaries, integrity contracts, cooperative async concurrency, and the standard library of Aipo.

---

## Manual Sections

- **[Syntax & Data Types](/en/manual/syntax-and-types)**: Primitives, literals, dynamic collections (`List`, `Dict`, `Set`, `Sequence`), binary `Bytes`, and operators.
- **[Control Flow & Failures](/en/manual/control-flow)**: Conditional branches, deterministic loops, explicit `fail`, and the transactional `attempt ... failed` block.
- **[Functions, Closures & Lambdas](/en/manual/functions-and-closures)**: Function declarations, named arguments, default values, anonymous functions, concise `=>` arrow syntax, and local functions with self-recursion.
- **[Interfaces & Contracts](/en/manual/interfaces-and-contracts)**: Structs, immutable `fixed` fields, `interface` with `satisfy`, `init` hooks, `invariant()` predicates, and signature contracts.
- **[Concurrency & Async](/en/manual/async-and-concurrency)**: `async fn`, sequential `await do ... end`, task combinators (`task.spawn`, `task.sleep`, `task.all`, `task.race`), and the virtual-time scheduler.
- **[Standard Library (Stdlib)](/en/manual/stdlib)**: Core modules (`math`, `random`, `json`, `encoding`, `path`, `url`, `regex`, `binary`, `time`, `testing`, `log`, `env`, `fs`).
- **[Packages & Modules](/en/manual/packages-and-modules)**: Project structure, `aipo.toml` manifests, deterministic lockfiles, offline consumption, and pinned GitHub dependencies.
