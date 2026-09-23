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
  `clamp`, trigonometry (`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`), logs/exp
  (`log`, `log2`, `log10`, `exp`), `hypot`, `sign`, `rad`, `deg`, plus constants `pi` and `e`.
- **`random`**: Bit-exact deterministic SplitMix64 PRNG (`random.create`, `random.seed`,
  `int`, `float`, `bool`, `choice`, `shuffle`).
- **`json`**: Canonical portable JSON parser and stringifier (`json.parse` rejecting
  duplicate keys, `json.stringify` with cycle detection).
- **`encoding`**: Canonical binary and text encodings (`base64_encode`, `base64_decode`,
  `base64url_encode`, `base64url_decode`, `hex_encode`, `hex_decode`, `utf8_encode`, `utf8_decode`).
- **`path`**: Logical and portable path manipulation (`join`, `normalize`, `is_absolute`,
  `basename`, `dirname`, `ext`).
- **`url`**: Canonical WHATWG URL model (`url.parse` returning structured dictionary with
  `href`, `origin`, `protocol`, `username`, `password`, `host`, `hostname`, `port`, `pathname`, `search`, `hash`).
- **`regex`**: Linear-time sandboxed regex engine (`regex.compile`, `regex.is_match`, `regex.replace`,
  `Pattern.is_match`, `Pattern.find`, `Pattern.find_all`, `Pattern.replace`, `Pattern.split`).
- **`string`**: Human-facing operations on extended grapheme clusters (`len`, `slice`, `graphemes`,
  `words`, `lines`, `casefold`, `encode_utf8`), search & inspect (`contains`, `starts_with`, `ends_with`,
  `find`), transformation (`lower`, `upper`, `capitalize`, `reverse`, `trim`, `split`, `join`, `replace`, `format`).
- **`collections`**: Shared eager and lazy vocabulary across `List`, `Dict`, `Set`, and `Sequence`
  (`map`, `filter`, `flat_map`, `find`, `find_index`, `any`, `all`, `count`, `reduce`, `first`, `first_or`,
  `last`, `last_or`, `take`, `skip`, `distinct`, `zip`, `chain`, `chunk`, `window`, `enumerate`, `entries`, `lazy`).
- **`binary`**: Primitive numeric serialization and deserialization on `Bytes`
  (`read_i8`, `read_u8`, `read_i16_le`, `read_i16_be`, `read_u16_le`, `read_u16_be`, `read_i32_le`,
  `read_i32_be`, `read_u32_le`, `read_u32_be`, `read_i64_le`, `read_i64_be`, `read_u64_le`, `read_u64_be`,
  `read_f32_le`, `read_f32_be`, `read_f64_le`, `read_f64_be`, matching `write_*` operations,
  unsigned LEB128 `read_varint`, `write_varint`, and `slice`).
- **`time`**: Capability-gated host clocks (`time.now`, `time.monotonic`) and portable pure calendar
  types (`Date`, `TimeOfDay`, `DateTime`, constructors `time.date`, `time.time_of_day`, `time.date_time`,
  `time.duration`, ISO 8601 parsers `parse_date`, `parse_time`, `parse_iso`, formatters `to_iso`,
  and `DateTime.epoch_seconds()`).
- **`testing` & `expect`**: Canonical assertion primitives (`equal`, `not_equal`, `true`, `false`, `none`,
  `some`, `failure`, `contains`, `approx`) returning `none` on success or recoverable `Failure` on mismatch.
- **`log`**: Structured level-based logging (`trace`, `debug`, `info`, `warning`, `error`) with
  configurable output sink routing to `io` output sink by default.
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
