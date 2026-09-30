# Evidence — P00-G15 / Static Contracts and Interface Conformance

**Goal:** `P00-G15` — pre-execution contract reporting, structural conformance for interfaces, and
the remaining ADP-001 questions
**Phase:** P00 (Foundation) · **Recorded:** 2026-09-15
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.

## Why this goal existed

`P00-G14` closed the two runtime gaps the S11 corpus found and left three residual limits in writing:
`aipo-sema` did not use the written annotations for a pre-execution report, an interface contract was
accepted blindly instead of being checked, and the ADP-001 questions on inverted `clamp` bounds,
slice saturation and NFC construction boundaries were still open. This goal closed all three.

| Residual | Canon decision source | Outcome |
|---|---|---|
| `aipo-sema` ignored the written annotations | Language Reference §4: "uma incompatibilidade comprovável é diagnóstico antes da execução; uma violação descoberta somente em runtime é contract fault" | **closed** |
| An interface contract was accepted instead of checked | Canonical Syntax, *Interfaces e `satisfy`*: "interfaces continuam estruturais e `satisfy` continua uma promessa/verificação explícita" | **closed** (with a real defect corrected) |
| ADP-001 Q3 — inverted `clamp` bounds | Language Reference §4 (value out of range → `Failure`) vs. index out of range → fault | **closed** — recoverable `Failure` |
| ADP-001 Q4 — slice outside the range | `Aipo Language — Especificação Viva`, `List` model: "slices fora da faixa são tolerantes/clamped, ao contrário de índices exatos" | **closed** — tolerant for `List`, `String` and `Bytes` |
| ADP-001 Q5 — where NFC is applied | Language Reference / Canonical Syntax: `String` is NFC "antes de ser exposta ao programa" | **closed** — at every construction boundary |

## Implementation

| Stage | Change |
|---|---|
| `aipo-sema` | `DeclaredContract` collects parameter contracts; `check_argument_contracts` matches positional and named arguments against the declaration; `check_return_contract` runs where the `return` is written; `literal_violates_contract` fires only for a provable mismatch (a literal against a core-type contract, or `none` against a non-nullable one) |
| `aipo-diagnostics` | New code `AIPO_SEM_CONTRACT_VIOLATION_STATIC` (language failure, exit code 1) |
| `aipo-ir` | `interface_operations` now emits the **caller-visible** arity (the receiver is not an argument at a call site), so the contract the runtime enforces is the one the interface declares |
| `aipo-vm` | `value_diagnostic_name` reports a struct's declared type name in contract diagnostics instead of the generic runtime kind |
| `aipo-lexer` | String-literal decoding normalizes to NFC (all prefixes, multi-line strings and identifiers) |
| `aipo-vm` | `String(value)` conversion and `String + String` (which interpolation lowers to) normalize to NFC |
| `aipo-stdlib` | `lower`, `upper`, `capitalize`, `replace`, `join` and `format` normalize their output; `reverse` already did |
| `aipo-cli` (tests) | The conformance harness serializes *every* in-process CLI invocation, not only the capturing one |

### The interface contract defect this goal found

The structural check landed in `P00-G14`, but it was wrong in the way that matters: the builder
counted the interface arity **including the receiver** (`fn draw(self)` → 1) while the runtime
compared the arity a call site sees (0). Every value failed its interface contract — including the
conforming ones — and the fault message named the generic runtime kind:

```
before — `render(Circle{r = 3})` on a conforming value
error: [AIPO_RT_TYPE_MISMATCH] contract violation at parameter `item`:
       expected Drawable.draw/1, got struct.draw/0
```

Both sides now use the caller-visible arity and the failing type's declared name:

```
after — conforming value runs, non-conforming value faults clearly
$ aipo run docs/conformance/programs/16_interface_contracts.aipo
player ana
ana hp 6
player ana
nothing to draw

$ aipo run docs/conformance/diagnostics/17_runtime_interface_contract.aipo
circle 3
error: [AIPO_RT_TYPE_MISMATCH] runtime fault [AIPO_RT_TYPE_MISMATCH]: contract violation at
       parameter `item`: expected Drawable.draw/0, got Blank without 'draw'

$ aipo run docs/conformance/diagnostics/18_runtime_interface_arity.aipo
error: [AIPO_RT_TYPE_MISMATCH] runtime fault [AIPO_RT_TYPE_MISMATCH]: contract violation at
       parameter `item`: expected Drawable.draw/0, got Odd.draw/1
```

### The conformance-harness race this goal found

Regenerating snapshots (`AIPO_UPDATE_SNAPSHOTS=1`) produced a polluted `programs/07_strings_and_math.stdout`
with a leading `3` the fixture never prints. The `io` sink is process-wide and the old harness only
serialized the *capturing* run, so another test executing a program concurrently wrote its output
into the capture buffer. The lock now covers every CLI invocation, and the regenerated snapshot
matches a direct `aipo run`. Three consecutive suite runs are green.

## Gate: `fmt` / `clippy`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_EXIT=0
(warnings: 0)
```

## Gate: `test`

```
$ cargo test --workspace
PASSED=146  FAILED=0

$ cargo test -p aipo-cli --test conformance
test result: ok. 13 passed; 0 failed
```

New certification:

| Artifact | Proves |
|---|---|
| `docs/conformance/diagnostics/15_sem_contract_violation.aipo` | a literal argument that cannot satisfy a written parameter contract is `AIPO_SEM_CONTRACT_VIOLATION_STATIC` before execution |
| `docs/conformance/diagnostics/16_sem_return_contract.aipo` | the same for a literal returned against `-> T` |
| `docs/conformance/diagnostics/12` / `13` | a mismatch the analyzer cannot prove still stays a runtime contract fault |
| `docs/conformance/programs/16_interface_contracts.aipo` | an interface as a written contract: `satisfy`, structural conformance at runtime, `T?` accepting `none`, an operation with an argument |
| `docs/conformance/diagnostics/17_runtime_interface_contract.aipo` | a value without the operation is a contract fault naming the struct |
| `docs/conformance/diagnostics/18_runtime_interface_arity.aipo` | a same-named operation of the wrong caller-visible arity is a contract fault |
| `docs/conformance/programs/17_unicode_nfc.aipo` | NFC at every construction boundary: escaped literal, concatenation, interpolation, `join`, `replace`, `format`, case mapping, `reverse`, and a mark with no precomposed form surviving |
| `docs/conformance/programs/18_tolerant_slices_and_clamp.aipo` | tolerant slices for `List`, `String` and `Bytes`; `clamp` with inverted bounds recoverable through `or_else`; mixed numeric promotion |
| `docs/conformance/diagnostics/19_runtime_clamp_inverted_bounds.aipo` | an unhandled inverted-bounds `Failure` (`AIPO_RT_FAILURE_UNCAUGHT`) |
| `crates/aipo-lexer/src/lib.rs::test_string_literals_are_nfc_normalized` | the lexer stores a decoded escape pair composed, and keeps raw escapes raw |
| `crates/aipo-vm/tests/data_and_errors.rs::test_string_concatenation_preserves_nfc` | `String + String` normalizes before the program can observe it |
| `crates/aipo-stdlib/tests/stdlib_tests.rs::test_string_operations_preserve_nfc` | `join`, `replace`, `format` normalize, `slice` preserves, and an uncomposable mark survives |
| `crates/aipo-vm/tests/data_and_errors.rs::test_interface_contract_accepts_a_conforming_operation` | a value exposing the operation satisfies the interface |
| `crates/aipo-vm/tests/data_and_errors.rs::test_interface_contract_names_the_failing_struct` | the fault names `Drawable.draw/0` and `Blank` |

Corpus size after this goal: 18 program fixtures, 19 diagnostic fixtures, 8 formatting fixtures and
3 module cases.

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
DOC_EXIT=0
(warnings: 0)
```

## Gate: `documentation_impact`

- `docs/adp/ADP-001-byte-and-core-types-as-values.md` — Q3, Q4 and Q5 closed with the canon source,
  the decision and the certifying fixture; the document is now fully resolved.
- `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` — G3 records interface structural
  conformance (and the arity defect that was corrected); the residual-limits list no longer claims
  interface contracts are unchecked.
- `docs/conformance/README.md` — matrix and inventory extended; the "verified gaps" section now lists
  the three residuals as closed.
- `docs/stdlib/mvp-subset.md` — interface contracts, static contract reporting, `clamp` bounds, the
  slice rule and a dedicated *String and Unicode normalization* table.
- `CHANGELOG.md`, `PROJECT_STATE.md`, `docs/PRUMO.md`.

## Known limitations (recorded, not hidden)

- **The static report covers provable mismatches only.** A literal against a core-type contract, and
  `none` against a non-nullable one, are provable from the expression alone. A variable, a call
  result or a `struct`/interface name needs the runtime, which stays the fault channel at the call
  boundary. `aipo-sema` is not a type checker; it reports contradictions it can prove.
- **A contract that names an interface is checked by name and arity, not by signature.** Canon makes
  interfaces structural, and this checks that the value exposes the declared operations with the
  declared caller-visible arity. A matching name with an incompatible parameter type is still only
  discovered when the operation runs.
- **NFC is not applied when reading the bytecode constant pool.** The lexer is the only producer of
  program-visible string constants, so normalizing there once is enough; this becomes a real
  decision again if a host text-input boundary or `Bytes.decode()` enters the MVP.
- **`string.slice` and `List[start..end]` share the tolerant rule, indexing does not.** Canon states
  the tolerant rule for slices and the faulting rule for exact indices; the extension to `String`
  and `Bytes` slices is the documented decision in ADP-001 Q4, not a canon quotation.
