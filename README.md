# Aipo

Aipo is a small, general-purpose, dynamically and strongly typed programming language
with optional signature contracts. Rust-first implementation: source compiles to
bytecode executed by a stack VM, with a JavaScript (ESM) backend proving parity over
the same conformance corpus.

```aipo
fn greet(name)
    return f"Hello, {name}!"
end

io.println(greet("world"))
```

## Quickstart

Prerequisites: Rust 1.85+ (see `rust-version` in `Cargo.toml`) and Node ≥ 20 for the
JavaScript backend tests.

```bash
cargo build -p aipo-cli
cargo run -q -p aipo-cli -- run examples/01_fizzbuzz.aipo
cargo run -q -p aipo-cli -- check path/to/program.aipo
cargo run -q -p aipo-cli -- build path/to/program.aipo --out dist && node dist/app.js
cargo run -q -p aipo-cli -- fmt path/to/program.aipo [--check]
```

Exit codes are contractual: `0` success, `1` language failure (diagnostic, runtime
fault, uncaught `Failure`, formatting drift), `2` usage error. Machine-readable
diagnostics: `--message-format=jsonl`. Full reference: `docs/reference/cli.md`.

## Language highlights

- Literals: `none`, `Bool`, `Int` (±(2^53−1)), finite `Float`, `Byte`, NFC `String`
  (`"…"`, `f"…"`, `r"…"`, `fr"…"`, `"""…"""`)
- `let`/`var`, closures with shared `var` capture, local `fn`, default/named args,
  trailing `do … end` blocks, pipelines (`|>`)
- `struct` with `fixed` fields, `init` + `invariant()` hooks, `impl` methods
- Ordered `List`/`Dict`, `Bytes(count)`, ranges and tolerant slicing
- Two error channels, never mixed: recoverable `Failure` (`fail`, `or_else`,
  `attempt … failed … end`) vs. runtime faults (contracts, bounds, overflow)
- Signature contracts (`name: Type`, `name!: Type`, `-> T`, `T?`) checked statically
  when provable and at runtime otherwise; structural interfaces + `satisfy`
- Modules: one file = one module, `import`/`export`, acyclic, init-once
- Minimal stdlib: Prelude (`len`, `copy`, `same`, `some`, `fail`, conversions),
  `math`, `string`, `io`

The executable definition of the language is the conformance corpus under
`docs/conformance/` (programs + snapshots, diagnostics + codes, formatter goldens,
module cases). Divergence between the VM and JS backends is a bug.

## Repository layout

```text
crates/            # workspace crates (compiler pipeline, VM, stdlib, CLI, JS backend)
  aipo-source, aipo-diagnostics, aipo-lexer, aipo-ast, aipo-syntax,
  aipo-hir, aipo-sema, aipo-ir, aipo-bytecode,
  aipo-vm, aipo-runtime, aipo-stdlib,
  aipo-formatter, aipo-cli, aipo-js
docs/              # canonical documentation
  canon/           # language authority (Language Reference, Syntax, Specs…)
  conformance/     # executable corpus (the language referee)
  waves/           # wave plans and exit gates
  evidence/        # per-goal proof records
  adp/             # architecture decision records
  reference/       # CLI reference, diagnostics catalog
examples/          # sample programs
```

The project is governed by **Prumo v0.6** (human-agent collaboration with Lean
Progressive Context): `prumo.json` is the canonical manifest, `ENTRYPOINT.md` the
context router, `PROJECT_STATE.md` the operational state. See `CONTRIBUTING.md`
before opening a change.

## Testing

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace          # unit + conformance + differential + fuzz (needs Node ≥ 20)
cargo doc --workspace --no-deps
prumo validate . && prumo doctor .
```

## Contributing

Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) — it covers the goal-driven workflow,
the no-invention policy (open ADPs, don't guess semantics), quality gates, and the
documentation delta rules.

## License

Licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
