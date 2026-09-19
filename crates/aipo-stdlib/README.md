# aipo-stdlib

Canonical standard library and Prelude V1 implementation for the Aipo programming language.

The closed surface implemented here is specified in [`docs/stdlib/mvp-subset.md`](../../docs/stdlib/mvp-subset.md).
Open semantic questions found while implementing it are recorded in
[`docs/adp/ADP-001-byte-and-core-types-as-values.md`](../../docs/adp/ADP-001-byte-and-core-types-as-values.md).

## Overview

- **Prelude V1**: globally available values (`none`, `true`, `false`), functions
  (`len`, `copy`, `same`, `some`, `fail`) and the explicit core-type conversions
  (`Int`, `Float`, `Byte`, `String`).
- **`math`**: `abs`, `min`, `max`, `floor`, `ceil`, `round`, `truncate`, `sqrt`, `pow`,
  `clamp`, plus the constants `pi` and `e`.
- **`string`**: `len`, `byte_len`, `contains`, `starts_with`, `ends_with`, `find`, `lower`,
  `upper`, `capitalize`, `reverse`, `trim`, `split`, `join`, `replace`, `slice`, `format`.
- **`io`**: `print` and `println`, with a pluggable capture sink for tests and embedders.

`register_stdlib(&mut vm, &mut registry)` installs the Prelude globals, the module
dictionaries and the `NativeRegistry` metadata in one call.

## Architecture

- Portable contracts only; no target-specific host leakage.
- Text operations work on Unicode code points; `byte_len` exposes UTF-8 bytes, and
  `reverse` works on extended grapheme clusters and re-normalizes its result to NFC.
- Strict argument rules straight from canon: `split`/`replace` reject empty patterns,
  `join` refuses implicit textual coercion, strict text parsing in conversions.
- Two error channels, never mixed: recoverable `Failure` values (Model B) for domain
  problems, `VmFault` for programming and contract violations. No Rust panic escapes.
- Dependencies: `aipo-runtime`, `aipo-vm`, `aipo-diagnostics`, plus the canon-preferred
  Unicode providers `unicode-segmentation` (grapheme clusters) and `unicode-normalization`
  (NFC).
