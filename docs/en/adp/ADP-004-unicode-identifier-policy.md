# ADP-004 — Unicode Identifier and Security Policy

**Status:** draft (open questions — characterization only, no restrictions added)
**Related:** `crates/aipo-cli/tests/unicode_security.rs` (pins current behavior),
`docs/evidence/P01-G02-*.md`, UAX #31, UTS #39 (consulted, not adopted)
**Authority:** subordinate to the Language Reference and the no-invention policy

## Verified current behavior (measured, then pinned in tests)

- Identifier characters are Rust `char::is_alphabetic` (first) and
  `is_alphanumeric` (rest), plus `_`. Consequence: precomposed non-ASCII letters
  work (`é`, `中`, `α`, Cyrillic); combining marks, zero-width characters and
  bidi controls are rejected with `AIPO_LEX_UNEXPECTED_TOKEN`.
- The source loader does **not** NFC-normalize: a decomposed `e` + U+0301
  identifier is rejected instead of composing to `é`. (String *literals* are
  normalized by the lexer per ADP-001 Q5; identifiers are not.)
- Mixed scripts and confusables are unrestricted: Cyrillic `сount` and Latin
  `count` coexist as distinct bindings with no warning.
- U+00A0 (no-break space) is not source whitespace: a trailing NBSP is a lexer
  error, not a skipped separator.
- Emoji/ZWJ sequences are preserved in strings; `len` counts code points.

## Open questions (all undecided — do NOT change behavior here)

1. Should the loader NFC-normalize (making decomposed identifiers compose), or
   is rejection the specified behavior?
2. Are confusable/mixed-script identifiers a risk worth a warning (UTS #39
   highly-restrictive/confusable detection), or is unrestricted acceptance the
   V1 position?
3. Is U+00A0-as-whitespace rejection intended, or should it join the skippable
   whitespace set?
4. Do zero-width/bidi rejections deserve their own diagnostic code, or is
   `AIPO_LEX_UNEXPECTED_TOKEN` the right (if generic) signal?
5. Should string *contents* face any restriction (they currently face none)?

## Non-goals of this ADP

- Adding any restriction, warning, or normalization without a follow-up design
  decision. These tests characterize; a policy change needs its own goal with
  canon sponsorship and migration of the corpus/fixtures it affects.
