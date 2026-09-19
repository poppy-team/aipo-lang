# Aipo — Testing Strategy

**Status:** normative
**Scope:** validation levels, gates, fixtures, evidence expectations for the Aipo implementation
**Blocking question:** which checks block completion?

## Test levels

1. **Unit** — per-crate domain logic (lexer classification, scope rules, bytecode verifier).
2. **Snapshot/golden** — tokens, syntax, HIR, IR, bytecode disassembly, formatter output,
   diagnostics. Committed under each crate's `tests/snapshots/` or `docs/conformance/`.
3. **Property** — lossless round-trip, formatter idempotency, lexer span slicing,
   parser never-panic.
4. **Integration** — end-to-end: `.aipo` program → `aipo run` → expected stdout/exit;
   `aipo check` → expected diagnostics.
5. **Fuzz** — pipeline smoke fuzz over arbitrary bytes; parser/lexer fuzz targets.
   Every crash finding becomes a regression fixture.
6. **Differential (Wave 2)** — VM ↔ JavaScript on the proven subset.

## Quality gates (blocking)

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo doc --workspace --no-deps
prumo validate . && prumo doctor .
```

Risk-gated (when relevant code changes): `cargo miri test` (no Wave 1 unsafe expected),
fuzz targets, `cargo audit`/`cargo deny`.

## Fixtures

- `docs/conformance/fixtures/pass/*.aipo` — must run/compile clean.
- `docs/conformance/fixtures/fail/**/*.aipo` — must produce the documented diagnostic code.
- Expected diagnostics live next to fixtures as `*.expected.jsonl`.

## Evidence expectations

| Claim | Evidence |
|---|---|
| "MVP runs" | integration test + fixture program output |
| "diagnostic correct" | fixture with expected code/span |
| "formatter stable" | idempotency test |
| "no panics leak" | fuzz smoke + panic-free test suite |
| "wave complete" | gate checklist in `docs/waves/` with links to evidence |
