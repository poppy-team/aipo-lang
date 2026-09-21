# Wave 3 — Async: Task, Scheduler, Await

**Status:** normative (process level)
**Authority:** canon Waves model (Wave 3), Fechamento §6, Stdlib canon
(Task/Sequence/Time), Poppy Pivot (async direction + await-do)
**Scope:** value layer, stdlib async surface, scheduler + `async`/`await`/`await do`
**Slices:** P02-G01 (tipos/valores) → P02-G02 (stdlib) → P02-G03 (infra async)

## Objective

Ship cooperative, deterministic async: `Task` values, a single-threaded
scheduler with virtual time, `async fn`/`await`/`await do`, structured
combinators (`all`/`race`/`timeout`/`cancel`/`group`/`spawn`), cancellation as
fault, with VM↔JS differential parity on ordering and cancellation.

## Non-negotiable rules (from canon)

- No implicit parallelism; FIFO run queue; depth-first await driving.
- `await do` is sugar for sequential awaits (never in subexpressions, never
  into nested fn/lambda/callback bodies, nested block form warns).
- `attempt` captures recoverable Failure only — never cancellation.
- Forgotten tasks are a static diagnostic.
- Divergence VM↔JS is a bug.

## Exit gate (evidence required)

- Corpus programs exercising tasks/sleep/all/race/timeout/cancel/group/await-do
  green on VM and Node with identical stdout, codes and exit status.
- Cancellation and await-cycle fixtures fault with committed codes.
- `cargo fmt/clippy/test/doc` green; no new `unsafe`; MSRV unchanged.
