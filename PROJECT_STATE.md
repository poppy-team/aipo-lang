# Current Project State

- Project: **aipo**
- Prumo: **0.6.0**
- Current phase: **P01 — JavaScript parity (Wave 2)**
- Active goal: **none — `P01-G01` DONE, next Wave 2 slice (performance/bench or `Bytes` packing) to be planned**
- Last completed goal: **P01-G01 — JS parity for the full MVP subset (DONE)**
- Completed slices:
  - **S1 (P00-G01)**: Workspace, `aipo-source`, `aipo-diagnostics`
  - **S2 (P00-G02)**: Lexer Core (`aipo-lexer`)
  - **S3 (P00-G03)**: Parser Core (`aipo-syntax`, `aipo-ast`)
  - **S4 (P00-G04)**: HIR Lowering (`aipo-hir`)
  - **S5 (P00-G05)**: Semantic Analysis (`aipo-sema`)
  - **S6 (P00-G06)**: Core IR & Bytecode (`aipo-ir`, `aipo-bytecode`)
  - **S7 (P00-G07)**: VM Core (`aipo-vm`)
  - **S8 (P00-G08)**: Data & Errors (`aipo-vm`)
  - **S9 (P00-G09)**: Runtime & Minimum Stdlib (`aipo-runtime`, `aipo-stdlib`)
  - **Backend Completion (P00-G10)**: pipeline gaps inside S1–S9 (`aipo-syntax`, `aipo-ir`, `aipo-bytecode`, `aipo-vm`, `aipo-stdlib`, `aipo-cli`)
  - **S10 (P00-G11)**: CLI & Formatter (`aipo-cli`, `aipo-formatter`)
  - **S11 (P00-G12)**: Conformance hardening (`docs/conformance`, MVP gate)
  - **MVP contract closure (P00-G13)**: construction hooks and `Bytes` (`init`, `invariant`, `Bytes(count)`)
  - **Runtime contract enforcement (P00-G14)**: `invariant()` at stable mutable boundaries (journal + rollback, recoverable `Failure`) and signature contracts at runtime (`AssertContract` faults)
  - **Static contracts & interface conformance (P00-G15)**: pre-execution `AIPO_SEM_CONTRACT_VIOLATION_STATIC`, runtime structural conformance for interface contracts (caller-visible arity, struct named in the fault), NFC at the `String` construction boundaries, and the closed ADP-001 decisions on `clamp` bounds and tolerant slices
  - **Wave 1 exit review (P00-G16)**: module-scope bindings visible inside `fn`/`impl` bodies and canonical local `fn` declarations (self-recursion via `FillSelfCapture`, shared `var` capture), certified by `programs/19_local_functions` and `programs/20_module_scope`; exit criteria audited against `docs/waves/wave-1-mvp.md`, gauntlet 100% (13/13, 11/11, 3/3, 146/146), non-delivery explicit (`Bytes` packing, `aipo-js`, `Set`/`Sequence`, LSP/REPL) — see `docs/evidence/P00-G16-wave-1-exit-review.md`
  - **JS parity MVP (P01-G01)**: `aipo-js` emitter + versioned shim + source maps, `aipo build`, differential suite green (20/20 programs, 19/19 diagnostics, 3 module cases, 152 workspace tests) — see `docs/evidence/P01-G01-js-parity-mvp-subset.md`
- Context methodology: **Lean Progressive Context (LPC)**
- Last updated: `2026-09-19T00:00:00Z`

## Next action

Wave 1 is closed. Wave 2 slice W2-1 is DONE (`docs/evidence/P01-G01-js-parity-mvp-subset.md`).
Remaining Wave 2 work (W2-2 diagnostics closure is already covered by the build suite;
next candidates are JS performance baselines or `Bytes` packing APIs) needs a new goal —
do not extend `P01-G01` silently.

## Recovery order

1. `ENTRYPOINT.md` or the platform adapter.
2. `prumo.json`.
3. `PROJECT_STATE.md`.
4. `docs/PRUMO.md`.
5. Active Goal under `.ai/goals/`.
6. Only relevant canonical docs/symbols/tests selected by the context strategy.

Do not load the entire repository by default.
