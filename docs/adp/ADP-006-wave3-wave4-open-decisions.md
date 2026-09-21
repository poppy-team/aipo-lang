# ADP-006 — Wave 3/4 Open Decisions (async, collections, packing, host)

**Status:** accepted (decisions below are implemented; follow-ups noted)
**Related:** Fechamento §6–§8, Stdlib canon (Task/Sequence/Time), Poppy Pivot
**Authority:** subordinate to canon; every item cites its canon source or is
explicitly marked as the smallest consistent choice.

## A. `Set` construction

Canon shows `Set` as a type with `.lazy()` but defines no literal or call form,
and V1 established that `List`/`Dict`/`Bytes` have no call form. Decision:
`Set(values: List) -> Set` conversion call (dedup, first-occurrence order,
insertion-ordered per canon). Rationale: mirrors the `Bytes(count)`/`Int(x)`
conversion-call pattern; no new literal syntax invented.

## B. `Sequence` sources and vocabulary

Canon diagram: `List / Dict / Set → .lazy()`. Decision: `.lazy()` exists on
List (elements), Dict (values), Set (insertion order). Range/String sources are
**not** added (unspecified). Vocabulary implemented: `map`, `filter`,
`flat_map`, `find`, `any`, `all`, `count`, `reduce` (initial value required),
`take`, `skip`, `group_by` (→ Dict), `distinct`, `zip` (→ List of pairs),
`chain`, `chunk` (→ List of Lists), `window` (→ List of Lists), `enumerate`
(→ List of `[index, value]`), `collect` (→ List). Laziness is a thunk chain;
`take` short-circuits so `take` over large sources stays bounded.

## C. Bytes packing

Canon names only storage formats (`i8…f64`), no API shape. Decision:
receiver-first methods `read_i8/u8/i16/u16/i32/u32/i64/u64/f32/f64(index)` and
`write_*(index, value)`, **little-endian** (game/binary convention; consistent
on VM and JS by construction), plus `String.encode()` (UTF-8 → Bytes) and
`Bytes.decode()` (UTF-8 → String, Failure on invalid). Exact-index violations
are faults (consistent with exact indexing); out-of-range writes never grow
the block silently.

## D. `Duration` and deferred clock types

Canon requires `Duration` plus `Date`/`TimeOfDay`/`DateTime` with ISO parsing,
but IANA DB belongs to a package (Wave 6). Decision: `Duration(seconds:
Int|Float)` conversion in Wave 3 with `+`, `-`, comparison and
`total_seconds()`; `Date`/`TimeOfDay`/`DateTime` deferred to Wave 6
`timezone` package (recorded, not hidden). `task.sleep(seconds)` accepts
Int/Float seconds directly; negative sleep is a recoverable Failure.

## E. Task combinators

Canon lists `sleep`, `all`, `race`, `timeout`, `cancel`, `group`, `spawn`
without signatures. Decisions: `task.spawn(fn)` and `task.spawn(fn, args:
List)`; `task.all(tasks: List) -> List` (fail-fast, cancels the rest, first
Failure wins); `task.race(tasks: List)` (first completion in deterministic
scheduler order wins, losers cancelled; empty list → Failure); `task.timeout
(task, ticks: Int)` (value or `Failure("timeout")`, task cancelled on expiry);
`task.cancel(task)` (marks cancelled, effective at suspension points);
`task.group()` → Group with `group.spawn(task)` / `group.wait()` (all
semantics; leftovers cancelled after fail-fast). `sleep(0)` is a reschedule
point. Ticks are u64 virtual time — fully deterministic, no wall clock.

## F. Await diagnostics (new codes)

Canon mandates the restrictions; code choice is implementation detail:
`AIPO_SEM_AWAIT_IN_SUBEXPRESSION` (explicit `await` outside statement /
initializer / return position), `AIPO_SEM_FORGOTTEN_TASK` (known-Task value
discarded without await/group combinator), `AIPO_SEM_NESTED_AWAIT_DO`
(redundant nested `await do`), `AIPO_SEM_PARAMETRIC_CONTRACT` (`Task[T]`
syntax until parametric contracts exist), `AIPO_RT_CANCELLED` (fault,
never capturable), `AIPO_RT_AWAIT_CYCLE` (fault: task awaiting itself,
directly or transitively).

## G. `Task[T]` parametric contracts deferred

Canon calls `Task[T]` a built-in contract but the language has no generics
(backlog: user generics out of V1). Decision: bare `Task` matches any Task
value at runtime; `Task[X]` in contract position is a dedicated diagnostic
pointing here (`AIPO_SEM_PARAMETRIC_CONTRACT`). Awaiting a provably non-Task
literal is a static contract violation (`AIPO_SEM_CONTRACT_VIOLATION_STATIC`,
certified by `docs/conformance/diagnostics/29_sem_await_literal.aipo`);
awaiting a non-Task at runtime is a contract fault
(`AIPO_RT_TYPE_MISMATCH`).

## H. Race/deadlock hazards closed by construction

No parallelism exists (single-threaded scheduler, FIFO run queue, depth-first
await driving, tick jump to minimum wakeup). `race` is order-deterministic by
design. Await cycles fault instead of hanging. Cancellation is checked at
suspension points only, so no inspect-and-kill race is possible.

## I. Wave 4 deviations recorded

- No `bevy` dependency: canon forbids *exposing* bevy/Ecs internals, and a
  100+ crate dependency for a demo adapter would violate the minimal-surface
  rule; `aipo-poppy` is self-contained and proves the ABI instead.
- **Host fault codes for restrictions canon mandates but does not name.** Canon
  fixes the ban (no use-after-free on a stale handle, no scoped binding escaping
  its callback) and names only `AIPO_RT_CAPABILITY_DENIED`; the code *names* are
  implementation detail, the same latitude §F recorded for the await
  diagnostics. Chosen: `AIPO_RT_STALE_HANDLE` and `AIPO_RT_SCOPE_ESCAPE`,
  alongside `AIPO_RT_CAPABILITY_DENIED`. A host value that cannot satisfy its
  declared contract is `AIPO_RT_TYPE_MISMATCH`, because canon classifies a
  contract violation discovered at a boundary as a type mismatch.
- **Capability paths are the tree.** Canon §11 lists a flat hierarchy including
  the `poppy.*` family, while the clock is treated as the single `clock`
  capability at the call site (`time.now`/`time.monotonic`). Both are the same
  rule once a path grants its descendants: `clock` covers `clock.wall` and
  `clock.monotonic`, and `poppy` covers every `poppy.<name>`. No wildcard syntax
  is introduced, and a profile that grants only `clock.wall` still cannot read
  the monotonic clock.
- Scoped-escape enforcement covers SetGlobal, Return, SetField, SetIndex,
  BuildList and BuildDict (every heap-publication point in the VM).
- `time.now`/`time.monotonic` require the `clock` capability (canon: clocks are
  capabilities); denial is `AIPO_RT_CAPABILITY_DENIED` fault. The default CLI
  script profile grants `clock`; tests construct denied hosts directly.
