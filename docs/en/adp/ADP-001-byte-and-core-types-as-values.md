# ADP-001 — Byte, `Bytes` and types as values in Prelude V1

**Status:** resolved — Q1 and Q2 resolved by `P00-G10`; Q3 (inverted `clamp` bounds), Q4
(slice saturation) and Q5 (NFC at construction boundaries) closed by `P00-G15`
**Related:** `docs/evidence/P00-G10-backend-completion.md`,
`docs/evidence/P00-G15-static-contracts-and-interface-conformance.md`,
`docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md`
**Origin:** Slice S9 (`P00-G09`), `crates/aipo-stdlib`, `crates/aipo-vm`
**Authority:** subordinate to `docs/canon/Aipo — Stdlib V1 Canônica e Contratos de Portabilidade…`,
`docs/canon/Aipo V1 — Language Reference…` and `docs/canon/Governança de Design e Evolução da Aipo…`
**Complements:** `docs/stdlib/mvp-subset.md`

The no-invention policy (`prumo.json` → `risk`, `docs/language/authority-map.md`)
requires that every material semantic decision without an answer in the canon become an open
question. This document gathers the ones slice S9 ran into. None of them blocks the MVP: for the
unambiguous parts, the implemented behavior is documented in `docs/stdlib/mvp-subset.md`.

## Q1 — `Byte` has no value kind of its own in the runtime

**Problem.** The canon treats `Byte` as a fundamental type (a compact integer `0..=255`) and defines
`Byte(value)` as an explicit, checked conversion: no wraparound and no silent saturation. The
VM value model (`aipo-vm::Value`) has no `Byte` variant, and none of the earlier layers
(lexer, AST, HIR, IR, bytecode) know the type.

**Current state.** `Byte(value)` validates the `0..=255` range from an `Int`, an integral `Float`,
or a `String`, and returns the value through the shared integer representation. Out of range, it
produces a recoverable `Failure`. This preserves the canon's mandatory check and introduces no
wraparound, but `Byte` and `Int` are currently indistinguishable at runtime.

**Question.** When does `Value::Byte(u8)` (or an equivalent managed type) land, and what does that
imply for: `type_name()` in diagnostics, equality (`Byte(1) == 1`?), `Byte -> Int -> Float`
promotion in arithmetic operations, and `Byte` literals in the lexer/AST and in the bytecode
constant pool?

**Out of scope for this slice.** Introducing the variant touches S2/S6/S7 and does not belong to S9.

**Resolution (`P00-G10`).** `Value::Byte(u8)` exists (alongside `Bytes` and `Type`), with its own
`type_name()`, structural equality with `Int`/`Float` (`Byte(1) == 1`), `Byte -> Int -> Float`
promotion in operations (`Value::widened`/`Value::needs_widening`), and literals
admitted by the conversions. `Int` and `Byte` are no longer indistinguishable at runtime.

## Q2 — Types as Prelude values (`List`, `Dict`, `Bytes`)

**Problem.** Prelude V1 (canon) lists `Int`, `Float`, `Byte`, `String`, `List`, `Dict`,
`Bytes` among the names available without an import. `Int`/`Float`/`Byte`/`String` have a
callable form defined by the canon (conversion). `List`, `Dict` and `Bytes` are used as type
names in contracts, but the canon defines no callable operation for them, and the runtime
has no *type value* kind.

**Current state.** `Int`, `Float`, `Byte` and `String` are registered as callable native globals.
`List`, `Dict` and `Bytes` are **not** registered: registering them would require inventing
semantics (what would `List(x)` do?) or inventing a type-value kind.

**Question.** What is the V1 model of *type values*: introspection (`T.name`; `x is T` already
exists via sema), generic construction (`List(x)`), or do types remain exclusively
contract notation with no runtime representation? And `Bytes`, which does not exist as a
runtime type in any layer — which slice does it land in?

**Resolution (`P00-G10`).** Types as values exist: `Value::Type(TypeTag)`, with `TypeTag`
enumerating `none`, `Bool`, `Int`, `Float`, `Byte`, `String`, `List`, `Dict`, `Bytes` and `Range`.
All the names are bound in the Prelude with identity, a canonical `name()` and structural
equality; `Int`/`Float`/`Byte`/`String` accept a call (conversion), whereas `List`/`Dict`/
`Bytes` are **only** type values — no call in V1 (generic construction was not invented).
The canonical way to construct `Bytes` remains open and is gap G4 of ADP-002.

## Q3 — `clamp` with inverted bounds

**Problem.** The canon defines `min`, `max` and `clamp` as numeric utilities and requires
`Int -> Float` promotion in mixed operations, but it does not define `clamp(value, min, max)` with
`min > max`.

**Current state.** Inverted bounds produce a recoverable `Failure` with an explicit message,
consistent with Model B (recoverable domain error) and without silently choosing
between "returns `min`" and "returns `max`".

**Question.** Is `Failure` the right channel, or is an inverted `clamp` a programming error
(fault)? Is there a canonical variant (`clamp_or`)?

**Resolution (`P00-G15`).** Recoverable `Failure`, and the answer comes from the criterion the canon
itself uses to separate the two channels: Language Reference §16 separates "recoverable failures" from
"programming runtime faults", and the canon classifies a **value outside a range** as a
`Failure` — `Int(value)`/`Float(value)`/`Byte(value)` with an out-of-range value produce
`Failure` "allowing recovery with `or_else`" (Language Reference §4), with "never wraparound or
silent saturation"; an **index outside the range**, on the other hand, is a fault (exact indexing), and reading/writing outside
the valid area of a `Bytes` is a "programming fault". `clamp` bounds are *values*, not structural
indices, so the coherent channel is `Failure`: `math.clamp(5, 3, 0) or_else -1` recovers, and an
unhandled `Failure` terminates the program like any recoverable failure. No variant
(`clamp_or`) was invented: the canon does not define one and the `or_else` form is already the canonical fallback.
Certification: `docs/conformance/programs/18_tolerant_slices_and_clamp.aipo` and
`docs/conformance/diagnostics/19_runtime_clamp_inverted_bounds.aipo`.

## Q4 — `string.slice` out of range

**Problem.** The canon requires an indexing runtime fault for `text[i]` out of bounds, but
does not fix the behavior of `slice` with out-of-range bounds.

**Current state.** Bounds are normalized (negative indices count from the end) and saturated
to the string's limits; `start >= end` returns `""`.

**Question.** Does slicing saturate (as implemented) or fail like indexing? If it saturates, is that
`String`-only or does it also apply to `List[start..end]`?

**Resolution (`P00-G15`) — asked and answered by the canon.** The question was already answered
for `List`: "Out-of-range slices are tolerant/clamped, unlike exact indices"
(`Aipo Language — Especificação Viva`, section `List` — decided fundamental model), together with
"Slicing uses `list[start..end]`, with an exclusive end bound, optional bounds and negative
indices". That a slice keeps creating a new value and that exact indexing remains a fault
are explicit rules of the same passage. The decision recorded here is that **the tolerant-slice rule
belongs to the slice category, not to a type**: it applies equally to `List`, `String` and `Bytes`, and
`string.slice(start, end)` follows the same normalization/saturation because it is the method form of the
same cut. `start >= end` produces an empty result in all of them.
Certification: `docs/conformance/programs/18_tolerant_slices_and_clamp.aipo` (bounds above the end,
below the start and inverted in `List`, `String` and `Bytes`), complementing
`docs/conformance/programs/09_slicing.aipo` (in-range slices) and
`docs/conformance/diagnostics/07_runtime_index_out_of_range.aipo` (an exact index is a fault).

## Q5 — NFC at construction boundaries

**Problem.** The canon establishes that `String` is UTF-8 normalized to NFC at the
construction/decoding boundaries, and that the result of `reverse()` obeys the invariants of
`String` (valid Unicode, UTF-8, NFC).

**Current state.** `string.reverse` re-normalizes the result to NFC (the only explicit
canon requirement on the result of an operation) using `unicode-normalization`.
The other operations (`lower`, `upper`, `replace`, `join`, `convert_string`, literals) do not
normalize, because global normalization is the responsibility of the construction boundaries
of the `String` model — not yet implemented in any layer.

**Question.** Where is NFC normalization actually applied: in the lexer (S2), in `String`
construction (VM), or in every operation that can break it? Which slice takes this on?

**Resolution (`P00-G15`) — at the boundaries, in every layer that constructs a `String`.** The canon
text the question cites ("automatically normalized ... before being exposed to the program")
does not name a layer, so the decision is the smallest one that satisfies the invariant without a
global sweep: normalize **where a `String` is constructed or where an operation can introduce
denormalization**. The three real points of introduction are now covered:

1. **Source-code literal** — the lexer decodes escapes and normalizes the content before
   creating the token (`crates/aipo-lexer`), which covers plain literals, multi-line literals, `f"..."` and
   module strings, and also identifier names.
2. **Conversion and concatenation** — `String(value)` normalizes on conversion, and `+` normalizes
   the concatenation, which is the operation where a base character can meet a combining mark
   (`f"..."` interpolation is lowered to `+`, so it inherits the rule).
3. **Stdlib operations that can join base + mark** — `lower`, `upper`, `capitalize`,
   `replace`, `join`, `format` and `reverse`. Purely substring operations (`slice`, `split`,
   `trim`) preserve the invariant for free: removing characters never makes two
   characters adjacent that were not before.

`==` remains structural equality over the stored value — the canon says NFC is an invariant
of `String`, not an operation executed in `==`. Nothing was normalized on the bytecode read path
(constant pool): normalizing there would be a per-access cost for an invariant already
guaranteed at the sole origin of literals. This is spelled out because it is the choice that may need
revisiting if a future input boundary (reading text from the host, `bytes.decode()`) enters the MVP
cut — today `io` has no input reading and `bytes.decode()` is outside the MVP.
Certification: `docs/conformance/programs/17_unicode_nfc.aipo` (literal with escape, concatenation,
interpolation, `join`, `replace`, `format`, `upper`, `reverse` and `capitalize`), plus the tests
`test_string_literals_are_nfc_normalized` (`crates/aipo-lexer/src/lib.rs`),
`test_string_operations_preserve_nfc` (`crates/aipo-stdlib/tests/stdlib_tests.rs`) and
`test_string_concatenation_preserves_nfc` (`crates/aipo-vm/tests/data_and_errors.rs`).

## Final decision

Closed. Q1 (`Byte` kind) and Q2 (type values) were resolved by `P00-G10`; Q3 (inverted
`clamp` bounds), Q4 (slice saturation) and Q5 (NFC at construction boundaries) were
closed by `P00-G15`, with the canon source cited in each section and the decision recorded in
`docs/stdlib/mvp-subset.md`. The `Bytes` construction gap (part of Q2) was reaffirmed as a
gate criterion in `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` and closed by
`P00-G13`. No semantics were invented: where the canon does not decide (inverted `clamp`), the decision is
derived from the canon's own explicit criterion for separating `Failure` from fault.
