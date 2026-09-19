# Evidence — P00-G11 / Slice S10 (CLI + Formatter)

**Goal:** `P00-G11` — CLI and Formatter: `aipo run`, `check`, `fmt`
**Phase:** P00 (Foundation) · **Slice:** S10 · **Recorded:** 2026-09-15
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.

## Deliverable

Two crates close the executable pipeline. Working backwards from the command surface in
`docs/reference/cli.md`:

| Crate | Responsibility |
|---|---|
| `aipo-formatter` | deterministic, idempotent rendering of a source file from its token stream |
| `aipo-cli` | argument parsing, the `source → syntax → HIR → sema → IR → bytecode → VM` orchestration, module resolution, diagnostic emission and exit codes |

The workspace now holds the 14 crates the Wave 1 plan calls for
(`cargo metadata` lists every crate under `crates/`).

### Command surface

```
aipo run <path> [--message-format=<human|jsonl>]
aipo check <path> [--message-format=<human|jsonl>]
aipo fmt <paths...> [--check]
aipo --version · aipo --help
```

Exit codes are part of the contract and every one is exercised by a test:

| Code | Meaning |
|---|---|
| `0` | the program ran to completion, `check` found no error, or `fmt` wrote/verified the files |
| `1` | language failure: any diagnostic of severity error, a bytecode verification failure, an uncaught runtime fault, or `fmt --check` drift |
| `2` | usage error: unknown command or flag, missing argument, unreadable file |

### Module resolution

`crates/aipo-cli/src/modules.rs` implements the S9 module model end to end, which the earlier
slices had only specified:

- one `.aipo` file is one module, resolved as `<name>.aipo` next to the importing file;
- `import m`, `import m: names`, `import m as alias` and `export` are all honored, and
  dependencies are loaded before dependents so initialization order is a topological one;
- **init once** holds under repeated imports (a module's top-level statements run once);
- privacy is enforced by name resolution, not by a separate pass: a non-exported declaration is
  renamed to `<module>::<name>` inside its own module, so an importer that names it gets an
  ordinary `AIPO_SEM_EXPORT_UNKNOWN`, and the same rejection applies to reaching it through the
  module namespace (`m.secret()`), which the namespace path would otherwise have allowed;
- cycles (`AIPO_SEM_IMPORT_CYCLE`) and missing modules (`AIPO_SEM_UNKNOWN_MODULE`) are reported
  as diagnostics rather than runtime faults.

## Gate: `fmt`

```
$ cargo fmt --all -- --check
(no output, exit 0)
```

## Gate: `check`

```
$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Gate: `clippy` (workspace lints deny warnings)

```
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```

## Gate: `test`

```
$ cargo test --workspace
total_passed=136 total_failed=0
```

S10-owned suites:

| Suite | Tests | What it pins |
|---|---|---|
| `aipo-cli/tests/conformance.rs` | 13 | program snapshots, failure fixtures, formatter drift, exit codes, JSONL, module cases |
| `aipo-cli/tests/fuzz_smoke.rs` | 3 | random bytes, mutated programs and truncated programs never panic the pipeline |
| `aipo-formatter` (`lib` + `tests/golden.rs`) | 6 + 4 | canonical rendering rules and idempotency over the formatting corpus |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
    Finished `dev` profile [unoptimized + debuginfo] target(s)
   Generated /home/raillen/Documentos/Projetos/aipo-lang/target/doc/aipo_ast/index.html and 14 other files
```

## Gate: `documentation_impact`

- `docs/conformance/README.md` — corpus layout, snapshot matrix, regeneration workflow.
- `docs/reference/cli.md` — already normative; every documented command, flag and exit code is
  now backed by a test rather than by intent.
- `CHANGELOG.md` — Wave 1 CLI and formatter entry.

## Known limitations (recorded, not hidden)

- `aipo fmt` renders from the token stream and normalizes whitespace, indentation and operator
  spacing. It does not reorder or rewrite syntax, and it does not yet implement `aipo fmt`
  over a directory or `*.aipo` glob beyond the paths given on the command line.
- Named arguments are resolved at the call site only when the callee is a declared function
  (see the S11 evidence record); a call through a stored function value keeps source order.
