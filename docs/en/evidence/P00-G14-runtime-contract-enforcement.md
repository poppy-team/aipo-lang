# Evidence — P00-G14 / Runtime Contract Enforcement

**Goal:** `P00-G14` — `invariant()` at mutation boundaries and signature contracts at runtime
**Phase:** P00 (Foundation) · **Recorded:** 2026-09-15
**Environment:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Proof attachments for the required gates. Commands are reproducible from the repository root.

## Why this goal existed

`P00-G13` closed three of the five gaps found by the S11 corpus and left two, because both needed a
runtime mechanism that did not exist yet: re-validating `invariant()` after a mutation, and checking
signature contracts at the call boundary. Both are MVP exit-gate criteria in
`docs/waves/wave-1-mvp.md`, so the gate could not be called complete before them.

| Gap | Canon decision source | Outcome |
|---|---|---|
| G2b — `invariant()` not re-evaluated after a mutation | Canonical Syntax: `candidate → provisional apply → verify → commit`; Interlúdio §3: validated at stable mutable boundaries, and a failed validation produces a `Failure` | **closed** |
| G3 — signature contracts not checked at runtime | Language Reference §4: `name: Type`, `name!: Type`, `-> T`, `T?`; a violation found only at runtime is a **contract fault** | **closed** |

The two error channels are different, and canon says so explicitly: an invariant failure at a
mutation boundary is a recoverable `Failure` ("a operação produz uma `Failure`"), while a contract
violation discovered at runtime is a programming fault that `attempt` cannot capture. No semantics
had to be invented for either.

## Implementation

| Stage | Change |
|---|---|
| `aipo-ir` | New `CoreInst::AssertContract { type_name, nullable, position }` and `CoreInst::CheckMutations`; `FnCtx` carries the function's return contract and the span of the frame's last field assignment |
| `aipo-bytecode` | New opcodes `AssertContract` (u16 name + u8 nullable + u16 position) and `CheckMutations`, wired through the emitter, the verifier (name-index bounds checks, nullable-flag check) and the disassembler |
| `aipo-vm` | `VmFault::ContractViolation`; a per-frame mutation journal (`MutationEntry`) with entry values; `SetField` applies provisionally and journals; `CheckMutations` verifies every participating instance, rolls back all of them and raises a recoverable `Failure`; `Vm::run` resolves each `Type.invariant` entry point from the module's function table; `CallFrame.journal_start` scopes the journal per frame |
| `aipo-stdlib` / `aipo-vm` | `TypeTag::from_name`, so a written contract name maps back to the core category the runtime already tests |

### Where the contracts are checked

- **Parameters**: in the callee's own prologue, after the default prologue, so a defaulted value is
  checked too and `init`, methods, plain functions and closures all get the check from one place.
- **Returns**: immediately before each `return expr`, on the value already on the stack. The
  instruction inspects the top of stack without consuming it, which is why one opcode serves both.
- **Invariant at mutation**: at the end of the enclosing mutable operation (every function/method
  return, including the implicit epilogue) and at every mutation statement of the module entry
  script, which has no enclosing operation to defer to. Canon forbids validating after each
  internal assignment, so the assignment itself only journals.

## Gate: `fmt` / `clippy`

```
$ cargo fmt --all -- --check
FMT=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s)
CLIPPY_EXIT=0
```

## Gate: `test`

```
$ cargo test --workspace
passed: 141  failed: 0
```

New certification:

| Fixture | Proves |
|---|---|
| `docs/conformance/programs/14_signature_contracts.aipo` | `Int` parameter, verified default, `T?` accepting `none`, the `Function` contract, a `struct` contract, a mutable `p!: Point` receiver and an `-> Int` return |
| `docs/conformance/diagnostics/12_runtime_contract_violation.aipo` | a parameter contract violation is a fault (`AIPO_RT_TYPE_MISMATCH`) |
| `docs/conformance/diagnostics/13_runtime_return_contract.aipo` | a return contract violation is a fault |
| `docs/conformance/programs/15_invariant_on_mutation.aipo` | commit at a method boundary, rollback with the entry value preserved on a method failure, the same for a direct assignment in the entry script, and rollback of two participating instances in one operation |
| `docs/conformance/diagnostics/14_runtime_invariant_mutation_uncaught.aipo` | an unhandled mutation invariant failure ends the program as a recoverable `Failure` (`AIPO_RT_FAILURE_UNCAUGHT`) |
| `crates/aipo-vm/tests/data_and_errors.rs::test_guarded_mutation_rolls_back_at_boundary` | the journal, the boundary verification and the rollback at the VM layer, with a host validator |
| `crates/aipo-vm/tests/data_and_errors.rs::test_signature_contract_violation_is_a_fault` | `AssertContract` raises `VmFault::ContractViolation`, not a `Failure` |
| `crates/aipo-vm/tests/data_and_errors.rs::test_nullable_contract_accepts_none` | `T?` accepts `none` |
| `crates/aipo-bytecode/src/lib.rs::test_compile_verify_and_disassemble_contracts_and_mutation_boundaries` | `AssertContract`, `CheckMutations` and `AssertInvariant` survive the emitter → verifier → disassembler round trip |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
(no warnings)
```

## Gate: `documentation_impact`

- `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` — G2b and G3 marked resolved with
  the decisions recorded, including the deviation note about *where* the mutation check runs.
- `docs/conformance/README.md` — snapshot matrix and fixture inventory extended.
- `docs/stdlib/mvp-subset.md` — signature contracts and invariant mutation boundaries documented as
  implemented behaviour.
- `CHANGELOG.md`, `PROJECT_STATE.md`, `docs/PRUMO.md`.

## Known limitations (recorded, not hidden)

- **Interface contracts are not checked at runtime.** A contract naming an interface is accepted,
  because the MVP has no runtime structural conformance check; inventing a failure for a contract
  the runtime cannot evaluate would be worse than the missing check.
- **Contract checks run in the callee prologue, not at the call site.** The observable behaviour is
  the same (a fault at the boundary, with the parameter name in the message), but a statically known
  mismatch is still reported at runtime rather than before execution; `aipo-sema` does not use the
  annotations yet.
- **`init` bodies still use the construction fault channel.** An invariant violated while
  constructing is `VmFault::InvariantViolation` (nothing may be published), while an invariant
  violated by a later mutation is a recoverable `Failure` (there is an entry state to restore).
  Both come from canon, from different paragraphs.
- **A `Failure` escaping an operation before its boundary keeps the applied value.** Canon only
  mandates a rollback when the *validation* fails, so entries are dropped without a restore when an
  unrelated failure unwinds the frame.
