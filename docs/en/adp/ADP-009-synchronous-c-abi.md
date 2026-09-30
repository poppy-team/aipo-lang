# ADP-009 — Synchronous, versioned C ABI

**Status:** accepted
**Date:** 2026-09-24
**Related:** `docs/adp/ADP-008-v0.1.0-language-release.md`, `docs/crates/crate-contracts.md`, `crates/aipo-host/`
**Authority:** subordinate to the canon; defines the first embedding boundary, not an engine.

## Decision

`aipo v0.1.0` exposes a synchronous, single-threaded, versioned C ABI. The ABI will be implemented in a separate crate, without exposing Rust structs, references, lifetimes or panics.

The internal `aipo-host` API remains the data contract. The C ABI is the adapter for external languages and engines.

## Minimum contract

- opaque runtime with explicit create/destroy;
- module loading and calling Aipo code;
- registration of host functions per module and capability;
- simple values copied by value;
- opaque handles for host-controlled objects;
- stable errors and sanitized text;
- documented ownership rules for every value and handle;
- versioned ABI to allow compatible evolution.

The first version does not create threads nor expose `Future`, async callbacks or an event loop of its own. The language's async semantics remain internal to the runtime; `step` and `run_until_idle` may be added in a future version.

## Safety and lifecycle

- The caller owns the runtime and the handles it sees.
- The host releases handles; the runtime never invents ownership for external references.
- A denied capability, a stale handle and a contract error become stable errors.
- No Rust error crosses the boundary.
- No dynamic module or `eval` enters the sandboxed profile.

## Mandatory proof

A headless C host must:

1. create and destroy a runtime;
2. load an Aipo module;
3. register a host function;
4. call the function with a simple value and a handle;
5. observe success, `Failure` and fault;
6. repeat the flow without panic, leak or invalid access.

The proof must have fixtures, documentation and an ownership test. Godot, engines and plugins are later consumers, not part of this ADR.

## Future evolution

Async will be an additive evolution of the ABI, not a silent change to v0. The future API must preserve synchronous execution and add only the pump/event loop operations the host needs.

## Out of scope

- Threads and `Send`/`Sync` in the VM.
- Dynamic library loading.
- `eval` or arbitrary memory access.
- Global module registry.
- Godot-specific or any engine-specific integration.
- Codecs, filesystem, assets or product-specific policies.
