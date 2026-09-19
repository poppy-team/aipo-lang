# Aipo

> **TL;DR:** Aipo is a small programming language. It runs on a bytecode VM written
> in Rust, and also compiles to JavaScript. Both backends must produce identical output.

```aipo
fn greet(name)
    return f"Hello, {name}!"
end

io.println(greet("world"))
```

## Start here (30 seconds)

You need: Rust 1.85+ and Node 20+.

1. Build it:
   ```bash
   cargo build -p aipo-cli
   ```
2. Run a program:
   ```bash
   cargo run -q -p aipo-cli -- run examples/01_fizzbuzz.aipo
   ```
3. Done. To learn the commands, see the table below.

## Commands

| What you want | Command |
|---|---|
| Run a program | `aipo run <file.aipo>` |
| Check for errors (no execution) | `aipo check <file.aipo>` |
| Build a JavaScript bundle | `aipo build <file.aipo> --out dist` then `node dist/app.js` |
| Format code | `aipo fmt <files...> [--check]` |
| Machine-readable errors | Add `--message-format=jsonl` |
| Version / help | `aipo --version`, `aipo --help` |

Exit codes: `0` = ok. `1` = language error. `2` = wrong command usage.

Full reference: [`docs/reference/cli.md`](docs/reference/cli.md).

## What is Aipo?

Short version, one idea per line:

- Small and general-purpose.
- Dynamically typed, but types are checked (strongly typed).
- Optional contracts on functions: `fn add(a: Int, b: Int) -> Int`.
- Safe strings: always valid Unicode, normalized.
- Two error channels, never mixed:
  - Recoverable problems → `Failure` (you catch with `attempt`).
  - Programming bugs → faults (they stop the program).
- Modules: one file = one module.
- Small standard library: Prelude, `math`, `string`, `io`.

<details>
<summary>More detail (types, structs, errors)</summary>

- Literals: `none`, `true`/`false`, `Int` (±(2^53−1)), finite `Float`, `Byte`, strings (`"…"`, `f"…"`, `r"…"`, `fr"…"`, `"""…"""`).
- `let`/`var`, closures, local functions, default and named arguments, `do … end` blocks, `|>` pipelines.
- `struct` with `fixed` fields, `init` and `invariant()` hooks, `impl` methods.
- Ordered `List`/`Dict`, `Bytes(count)`, ranges, tolerant slicing.
- Interfaces are structural, checked with `satisfy`.
- The executable definition of all of the above is the corpus in [`docs/conformance/`](docs/conformance/).
- If the VM and the JS backend disagree on any program, that is a bug.

</details>

## Where things live

| Path | What it is |
|---|---|
| `crates/` | The implementation (compiler, VM, stdlib, CLI, JS backend) |
| `docs/conformance/` | The test corpus — the language referee |
| `docs/canon/` | The language rules (what wins arguments) |
| `docs/waves/` | Plans per delivery wave |
| `docs/evidence/` | Proof that each goal was done |
| `docs/reference/cli.md` | Command manual |
| `examples/` | Sample programs |

## Contributing and governance

**To contribute, read [`CONTRIBUTING.md`](CONTRIBUTING.md) first.** The short rules:

1. No code without an explicit Goal.
2. Never guess language semantics — open questions become written decisions (ADPs).
3. Keep all quality gates green (see below).
4. Update docs together with code.

This project is governed by **Prumo v0.6** (human-agent collaboration with lean
context). The local entry points are:

- [`prumo.json`](prumo.json) — project manifest.
- [`ENTRYPOINT.md`](ENTRYPOINT.md) — context router.
- [`PROJECT_STATE.md`](PROJECT_STATE.md) — current state.
- [`docs/PRUMO.md`](docs/PRUMO.md) — documentation map.

> Prumo framework: [github.com/raillen/prumo](https://github.com/raillen/prumo).

## Quality gates

Run all of these. All must be green.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
prumo validate . && prumo doctor .
```

## License

Dual license, your choice:

- MIT — [`LICENSE-MIT`](LICENSE-MIT)
- Apache 2.0 — [`LICENSE-APACHE`](LICENSE-APACHE)
