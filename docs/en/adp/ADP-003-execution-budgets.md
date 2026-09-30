# ADP-003 — Execution Budgets for Untrusted Programs (fuel, memory, interruption)

**Status:** draft (open question — no semantics implemented, nothing decided)
**Related:** `docs/evidence/P01-G02-*.md` (resource-exhaustion suite), crate contracts
(`aipo-vm` Owns line corrected by the same goal), Fechamento Arquitetural §10–11
**Authority:** subordinate to `docs/canon/Aipo V1 — Language Reference…` and
`docs/language/authority-map.md` (no-invention policy)

## Verified facts (not decisions)

- The VM enforces exactly one execution budget today: the operand-stack depth
  limit (1024), surfacing as `AIPO_RT_OVERFLOW` with a recursion hint.
- There is **no** instruction/fuel budget, no allocation/memory accounting, and no
  in-VM interruption mechanism. A `loop … end` program runs until the host kills
  it; the test suite proves this with an external watchdog, not with a VM guarantee.
- The crate contract historically listed "VM-level fuel/debt accounting" under
  `aipo-vm` Owns. That line described an aspiration, not the implementation, and
  has been corrected to the stack-depth limit with a pointer here.
- `cargo-fuzz`/libFuzzer execution, Miri, and sanitizer runs are CI-tier
  evaluations, not current gates (see the gauntlet evidence record).

## Open questions (all undecided)

1. **Exhaustion signal:** when a budget trips, is the outcome a recoverable
   `Failure`, a runtime fault with a new diagnostic code, or process abort?
   Each choice changes the Failure/fault contract and needs Language Reference
   backing that does not exist yet.
2. **Budget scope:** per call, per module run, per host session? Who sets it —
   source syntax, CLI flags, or host API only?
3. **Memory accounting:** what counts (collectotas, closures, strings, bytecode),
   and is the accounting exact or sampled?
4. **Interruption:** cooperative (checked between instructions) or preemptive?
   What state is observable after an interrupt?
5. **JS backend:** any budget must have a defined VM↔JS parity rule; the shim
   currently has no budget either.

## Non-goals of this ADP

- Inventing fuel semantics inside a test/quality goal. Budgets change observable
  behavior and need their own design goal with canon sponsorship.
- Claiming sandboxing: Aipo output "is not by itself a security sandbox"
  (Fechamento §10). Untrusted scripts still require VM/host sandboxing or
  external isolation.

## Exit criteria

This ADP closes when a design goal specifies the signal, scope, accounting and
parity rule above, with fixtures proving each — or explicitly scopes budgets out
of V1 with a documented rationale.
