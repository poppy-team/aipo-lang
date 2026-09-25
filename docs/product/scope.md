# Aipo — Project Scope

**Status:** normative
**Scope:** in-scope capabilities, non-goals, compatibility constraints
**Blocking question:** What is explicitly out of scope?

## `aipo v0.1.0` language release

The first product boundary is a usable language release, not a complete platform. It must deliver:

- V1 language surface: syntax, modules, contracts, `Failure`/fault separation and the essential standard library.
- Async language surface: `async fn`, `await`, `task.*` and deterministic cooperative scheduling.
- Minimal CLI: `aipo run`, `aipo check`, `aipo build`, `aipo fmt` and `aipo test`.
- Rust embedding and a versioned, synchronous C ABI with opaque handles and explicit capabilities.
- Explicit, bounded JavaScript interoperability; no implicit Node or browser globals.
- One small real interoperability proof for each boundary: Rust, C and JavaScript.
- Local and pinned-GitHub packages, lockfile, cache and opt-in authentication already delivered by P04.
- Executable examples, documentation, migration notes and all repository quality gates.

The release does not promise a complete port of any external library. It proves that the boundary is safe, explicit and usable. The C ABI boundary is specified in `docs/adp/ADP-009-synchronous-c-abi.md`; the interoperability proof criteria are in `docs/adp/ADP-010-interoperability-thin-proofs.md`.

## Post-v1 product boundaries

The following remain product work after the language release:

- Godot and other engine adapters.
- The future Aipo/Petunia3D engine; its core design is intentionally deferred.
- `aipo new`, `aipo watch`, LSP, REPL, debugger and profiler.
- DOM/storage web profile, `fetch`, workers and WebAssembly.
- Registry, publication, SemVer solving, vendor trees and hot reload.
- Visual editors, lifecycle scripts and complete third-party framework ports.

## Current implementation status

The language, VM, JavaScript backend, async surface, host contracts, local/GitHub package path and cache verification are implemented. The C ABI, minimal test runner and the three interoperability proofs are the next implementation targets for `v0.1.0`.

## Compatibility constraints

- Implementation edition: Rust Edition 2024.
- Target platforms: Tier 1 Linux (x86_64, aarch64), macOS (Apple Silicon, Intel), Windows (x86_64).
- Safe Rust first: `unsafe_code = "forbid"` across workspace.
- No Rust panic may ever leak as an Aipo runtime error.
