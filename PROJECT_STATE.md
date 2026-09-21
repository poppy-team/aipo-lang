# Current Project State

- Project: **aipo**
- Prumo: **0.6.0**
- Current phase: **Wave 3 closed** (next: **P03 — Wave 4 host ABI/Poppy**)
- Active goal: **none** — `P02-G01`, `P02-G02` and `P02-G03` are DONE; Wave 3 is closed
- Last completed goal: **P02-G03 — Wave 3 infra: sintaxe async fn, await do e diagnósticos estáticos (DONE)**
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
  - **Deep quality gauntlet (P01-G02)**: performance baseline, fuzzing, property suites, supply-chain policy (`deny.toml`), and security boundaries — see `docs/evidence/P01-G02-deep-quality-gauntlet.md`
  - **Wave 3 tipos e valores (P02-G01)**: Set com ordem de inserção, Sequence lazy, packing de Bytes little-endian (`read_*`/`write_*`), Duration e Task/Group handles com paridade diferencial VM↔JS e conformance — ver `docs/evidence/P02-G01-wave3-types-and-values.md`
  - **Wave 3 combinadores assíncronos e scheduler (P02-G02)**: Scheduler cooperativo determinístico com tempo virtual, combinadores assíncronos (`task.spawn`, `task.sleep`, `task.all`, `task.race`, `task.timeout`, `task.cancel`, `task.group`), `await` opcode e paridade diferencial total VM↔JS — ver `docs/evidence/P02-G02-wave3-stdlib-async.md`
  - **Wave 3 infra assíncrona (P02-G03)**: `async fn` como protocolo de chamada em forma de topo, local, anônima e método de `impl` (receptor = argumento 0), `await do … end` sequencial, diagnósticos estáticos `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`/`AIPO_SEM_FORGOTTEN_TASK`/`AIPO_SEM_NESTED_AWAIT_DO`, contrato estático do operando de `await`, faults de runtime `AIPO_RT_AWAIT_CYCLE`/`AIPO_RT_CANCELLED` com paridade VM↔JS, e endurecimento de literais numéricos com helper único em `aipo-lexer::number`; conformance programas 24–27 e diagnósticos 20–29 (suíte 301 testes) — ver `docs/evidence/P02-G03-wave3-async-syntax-and-diagnostics.md`
- Context methodology: **Lean Progressive Context (LPC)**
- Last updated: `2026-09-21T05:05:00Z`

## Next action

Wave 3 is closed: `docs/waves/wave-3-async.md` records the met exit gate, and `P02-G01`/`P02-G02`/`P02-G03` are DONE. Open the Wave 4 phase from `docs/waves/wave-4-host-poppy.md` — first slice `P03-G01` (host embedding ABI), then `P03-G02` (Poppy adapter + demo). Open items still carried: `P00-G16`, `P01-G01` are REVIEWING and `P01-G02` is EXECUTING.

## Recovery order

1. `ENTRYPOINT.md` or the platform adapter.
2. `prumo.json`.
3. `PROJECT_STATE.md`.
4. `docs/PRUMO.md`.
5. Active Goal under `.ai/goals/`.
6. Only relevant canonical docs/symbols/tests selected by the context strategy.

Do not load the entire repository by default.
