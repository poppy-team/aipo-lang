# Aipo — Security and Trust Contract

**Status:** normative
**Scope:** trust boundaries, permission model, secrets model, recovery
**Blocking question:** Which actions require explicit approval?

## Trust boundaries

1. **Host vs Guest Boundary**: Aipo source code executed by the interpreter is untrusted user code. It must never cause crashes, undefined behavior, or unauthorized memory access on the host system.
2. **Compiler Frontend Boundary**: Source input is assumed potentially hostile (e.g. deeply nested ASTs, exponential expansions). The lexer and parser must enforce recursion limits and bounded allocations.
3. **Agent and Automation Boundary**: Automated code agents work within declared sandbox permissions.

## Permission model and approval requirements

### Which actions require explicit approval?
- Modifying repository governance policies or bypassing CI quality gates.
- Introducing dependencies containing `unsafe` Rust code.
- Altering published canonical diagnostic codes or grammar contracts.
- Executing destructive file-system operations outside target workspace directories.

### Approved dependency exceptions

| Dependency | Used by | Contains `unsafe` | Justification | Approved |
|---|---|---|---|---|
| `unicode-normalization` 0.1.25 | `aipo-stdlib` (`string.reverse` NFC re-normalization) | yes — `char::from_u32_unchecked` confined to Hangul decomposition in `src/normalize.rs` | Canon names this crate as the preferred Rust provider for NFC (`Aipo — Stdlib V1 Canônica …` → “Implementação Rust — referências preferidas”); canon requires the result of `reverse()` to keep the `String` NFC invariant; no workspace crate enables `unsafe` | 2026-09-15, explicit user approval during slice S9 (`P00-G09`) |

Dependencies added to the workspace must be scanned before use. For slice S9 the remaining additions were verified
unsafe-free: `unicode-segmentation` 1.13.3 (`#![deny(unsafe_code)]`) and its transitive `tinyvec` 1.13.3
(`#![forbid(unsafe_code)]`).

## Secrets model

- Zero secrets committed to version control.
- No environment variables containing credentials may be echoed to log outputs or test snapshots.

## Recovery

- In the event of a security violation or regression:
  1. Offending changes are reverted immediately.
  2. A reproducing regression fixture is committed under `docs/conformance/fixtures/`.
  3. Quality gate checks are re-run to confirm remediation.
