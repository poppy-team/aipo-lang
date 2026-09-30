# ADR 001: Architectural baseline and Clean Architecture principles

## Status
Partially superseded — see the note below. The generic content (SRP, DIP,
YAGNI) remains as guidance; the compiler architecture decisions
live in `docs/architecture/overview.md` + `docs/crates/crate-contracts.md`.

> Supersession note (2026-09-20, docs audit): this ADR was written
> as boilerplate (it mentions DTOs, persistence, drivers, "classes") and does not
> describe the real architecture — a frontend → Core IR → two backends pipeline,
> with no database, no web frameworks, no classes (Rust). Nothing here was
> deleted; where this document conflicts with `overview.md`/crate-contracts,
> those win (see `docs/language/authority-map.md`: architecture contracts
> take precedence as the source of the implemented state).

## Context
The project requires high maintainability, strict isolation of business rules from frameworks and external dependencies, and support for unit and integration tests without heavy infrastructure dependencies.

## Decision
We adopt Clean Architecture (Ports and Adapters). All dependencies must point toward the core domain. Communication with external infrastructure must occur exclusively through port interfaces.

## Consequences
- **Positive**: Complete in-memory testability; easy replacement of persistence drivers; framework independence.
- **Negative**: Introduction of intermediate data-mapping layers (DTOs and entities).
