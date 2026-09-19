# Prumo — .

This is the intent router for humans and agents. Add links as stable documentation is created; do not create empty documentation solely to populate this map.

## Current state

- [Project state](../PROJECT_STATE.md)
- `prumo.json` — canonical project configuration

## I want to use the product

Add user tutorials, how-to guides, reference and explanations under `docs/user/` as needed.

## I want to develop/contribute

Add onboarding, codebase tour, build/test/debug and task-oriented development guides under `docs/developer/`.

## I want to operate/support it

Add deployment, configuration, observability, runbooks, backup/recovery, troubleshooting and release guidance under `docs/operations/` / `docs/support/` as needed.

## I am an AI agent

1. Read the active Goal.
2. Use the smallest sufficient context.
3. Prefer structural/symbol/document-section pointers.
4. Expand only when evidence is insufficient.
5. Keep output bounded.
6. Update only impacted canonical docs.
7. Record evidence/intelligence and garbage-collect temporary context.

## Architecture / decisions / specs

- [Crate contracts](crates/crate-contracts.md) — per-crate responsibility, dependency and invariants
- [Wave 1 plan](waves/wave-1-mvp.md) — MVP subset, vertical slices, exit gate
- [Language authority map](language/authority-map.md) — which document wins, and the no-invention/evidence policies
- [CLI reference](reference/cli.md) — stable `aipo run/check/fmt` surface, machine output and exit codes
- [MVP stdlib subset](stdlib/mvp-subset.md) — implemented Prelude V1, `List`/`Dict` methods and `math`/`string`/`io` surface
- [Conformance corpus](conformance/README.md) — fixture layout, snapshot matrix, regeneration and the gauntlet rubric
- [ADP-001 — Byte and core types as values](adp/ADP-001-byte-and-core-types-as-values.md) — Q1/Q2 resolved, Q3–Q5 open
- [ADP-002 — Construction hooks and runtime contracts](adp/ADP-002-construction-hooks-and-runtime-contracts.md) — resolved: G1/G2/G4 by `P00-G13`, G2b/G3 by `P00-G14` (interface structural conformance corrected and certified by `P00-G15`)
- [ADP-001 — Byte, `Bytes` and core types as values](adp/ADP-001-byte-and-core-types-as-values.md) — resolved: Q1/Q2 by `P00-G10`, Q3/Q4/Q5 by `P00-G15`

Add further architecture, ADR/RFC and specifications as the project grows.

## Evidence

Gate evidence records live under `docs/evidence/`, one per completed goal:

- [P00-G09 — Runtime & minimum stdlib](evidence/P00-G09-slice-s9.md)
- [P00-G10 — Backend completion](evidence/P00-G10-backend-completion.md)
- [P00-G11 — CLI and formatter](evidence/P00-G11-cli-and-formatter.md)
- [P00-G12 — Conformance and MVP gate](evidence/P00-G12-conformance-and-mvp-gate.md)
- [P00-G13 — Construction hooks and Bytes](evidence/P00-G13-construction-hooks-and-bytes.md)
- [P00-G14 — Runtime contract enforcement](evidence/P00-G14-runtime-contract-enforcement.md)
- [P00-G15 — Static contracts and interface conformance](evidence/P00-G15-static-contracts-and-interface-conformance.md)
- [P00-G16 — Wave 1 exit review](evidence/P00-G16-wave-1-exit-review.md)
- [P01-G01 — JS parity for the full MVP subset](evidence/P01-G01-js-parity-mvp-subset.md)

## Goals

Goals live under `.ai/goals/<phase>/` and define measurable completion.

## Durable intelligence

Compact project/task intelligence lives in `.prumo/history/project-intelligence.json`.
