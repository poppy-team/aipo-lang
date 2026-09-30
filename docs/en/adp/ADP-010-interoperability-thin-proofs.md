# ADP-010 — Thin interoperability proofs for Rust, C and JavaScript

**Status:** accepted
**Date:** 2026-09-24
**Related:** `docs/adp/ADP-008-v0.1.0-language-release.md`, `docs/adp/ADP-009-synchronous-c-abi.md`
**Authority:** subordinate to the canon; defines the interoperability criterion for the language's first release.

## Decision

`aipo v0.1.0` does not promise to port complete libraries. It delivers a small, real, testable proof for each boundary:

- Rust: one real host module; egui can be the preferred proof.
- C: a small library with an explicit lifecycle and ownership.
- JavaScript: a module with an explicit bridge, with no implicit Node or browser globals.

The choice of library considers a clear API, compatible license, maintenance, ownership and testability. Popularity and size are not sufficient criteria.

## Common contract

Each proof must demonstrate:

- the surface exposed to the language;
- input and output conversion;
- ownership or handle lifetime;
- the required capability;
- error translation;
- observable behavior in the test;
- integration documentation and an executable example.

The proof must be small enough to reveal boundary problems and large enough to use a real library.

## Rust

The Rust proof must go through the host API path, not through a language-specific extension. A GUI integration can use egui as the proof, but the release does not require a complete wrapper nor an editor.

## C

The C proof must use the ABI from `docs/adp/ADP-009-synchronous-c-abi.md`. The library must have create/use/release or an equivalent lifecycle form. Simply calling a function with no errors or ownership is not enough.

## JavaScript

The JavaScript proof must use a named, limited bridge. The runtime must not implicitly expose `window`, `globalThis`, Node modules or `eval`. The choice between browser and Node is a profile decision, not part of the language's identity.

## Out of scope

- Porting entire frameworks or libraries.
- Node as a complete host language.
- DOM, `fetch`, workers or WebAssembly.
- Package registry or publishing.
- Engine integration.
- Implicit interop through globals or `eval`.

## Completion criterion

The release can declare interoperability complete when the three proofs have tests, docs, ownership, capabilities, errors and executable examples. Implementing additional libraries remains in separate goals.
