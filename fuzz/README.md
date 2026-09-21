# Fuzzing Aipo with libFuzzer (`fuzz/`)

Coverage-guided fuzzing for the frontend. Deterministic smoke fuzzing lives in
`crates/aipo-cli/tests/fuzz_smoke.rs`; this directory is the nightly tier.

## Prerequisites

- Nightly toolchain with `miri`-era components not required, but `cargo-fuzz`
  needs nightly cargo: run everything through the rustup shim with
  `~/.cargo/bin` **first** on `PATH` (the distro `/usr/bin/cargo` does not
  understand `+toolchain`, and `cargo-fuzz` resolves its inner `cargo` via
  `PATH`).
- Seed corpora are committed (`fuzz/corpus/<target>/`, seeded from
  `docs/conformance/` and `examples/`).

## Targets

| Target | Feeds | Asserts |
|---|---|---|
| `lexer_tokens` | Arbitrary bytes → source loader + lexer | No panic; spans valid; stream ends with `Eof` |
| `frontend_check` | Arbitrary bytes → parse + HIR + sema + Core IR | No panic (found the IR builder slot panic, fixed; regression: `programs/23`) |
| `formatter` | Arbitrary bytes → `format_text` | No panic |

## Run

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo +nightly fuzz run <target> fuzz/corpus/<target> -- -max_total_time=120
```

New `fuzz/artifacts/<target>/crash-*` files are evidence, not fixtures: minimize
the input, add the smallest deterministic regression (corpus program or unit
test), and record it in the evidence file. Never commit unminimized blobs as
regression tests.
