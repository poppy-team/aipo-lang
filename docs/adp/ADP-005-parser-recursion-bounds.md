# ADP-005 — Parser Recursion Bounds (Nesting Limits)

**Status:** accepted (implemented with this gauntlet; limits are robustness bounds)
**Related:** `AIPO_PARSE_NESTING_TOO_DEEP` (diagnostics catalog), `crates/aipo-syntax`
(depth guards + progress guarantee), `crates/aipo-cli/tests/resource.rs`,
`docs/evidence/P01-G02-*.md`
**Authority:** subordinate to the Language Reference (which defines no nesting
requirement) and the no-invention policy

## Verified facts (measured, not assumed)

- Before this change, hostile-but-well-formed nesting aborted the host process
  (`SIGABRT`, exit 134): `((((…))))` past ~500–800 levels and nested `if` past
  ~300–600 levels on an 8 MiB main stack. An abort is never canonical behavior:
  the testing strategy already requires that no Rust panic (and by extension no
  host crash) escapes as an Aipo user error.
- Measured per-level frame cost is ~10–27 KiB depending on the construct, so any
  bound must sit far below `2 MiB / 27 KiB ≈ 74` levels to also hold on 2 MiB
  test threads.
- The deepest nesting in the conformance corpus is single digits.
- A second, latent defect fell out of the same investigation: body loops
  (`while !terminator { parse_stmt … }`) assume every failed parse consumes
  input, so *any* non-consuming failure hangs them forever. The first version of
  the depth guard tripped exactly this path.
- A third shape evaded the in-parse guards entirely: trees built iteratively but
  deeply (a 5000-term `1 + 1 + …` chain, a 500-placeholder `f"…"`), which abort
  downstream recursive walkers (HIR lowering and beyond). The fix is an
  iterative AST depth post-pass in `aipo_syntax::parse` (`depth.rs`, exhaustive
  over every recursive AST shape so a new one is a compile error, not a hole).

## Decisions (all recorded here so nothing is silently invented)

1. **Bounds:** expression nesting 128, block nesting 64, AST depth 128. All are
   >15× the deepest corpus nesting and keep worst-case stack near ~1–2 MiB —
   inside the smallest supported host stack with margin. These numbers are
   implementation robustness bounds, **not** language semantics: every program
   below them parses exactly as before (proven by the unchanged test suite plus
   corpus snapshots).
2. **Signal:** one new diagnostic code, `AIPO_PARSE_NESTING_TOO_DEEP`
   (severity `error`), pointing at the offending token. Reusing
   `AIPO_PARSE_UNEXPECTED_TOKEN` would misdescribe the problem and hurt the
   diagnostic accessibility rubric; a dedicated code is the honest signal.
3. **Cascade control:** after the first overflow the parser sets an abort flag —
   the rest of the file is skipped quietly instead of emitting one error per
   remaining token (a 600-deep file previously produced ~1000 follow-ups).
4. **Progress guarantee:** a `parse_stmt` failure that consumed nothing advances
   one token. This path provably never fires on previously-terminating inputs
   (they always made progress, otherwise they would already hang), so observable
   recovery behavior there is unchanged by construction.

## Non-goals

- No general recursion-limit syntax, no configurable limit flag, no statement
  about what "should" nest deeply. If canon ever requires deeper nesting, the
  bound moves with evidence, not by editing around it.
- HIR/sema lowering recursion was measured (300-deep `if` lowers fine) but is
  not separately guarded; the parser bound caps everything downstream. If a
  future construct recurses outside the parser, it gets its own ADP entry.
