# Development Trajectory

This section documents the complete engineering journey, architecture evolution, and milestones of the **Aipo** programming language, from the initial tokenization concepts to the consolidation of release v0.1.0.

Every wave and vertical slice was built under the **Lean Progressive Context (LPC)** methodology and governed by the **Prumo CLI**, with strict acceptance criteria, exhaustive automated test coverage, and zero unverified assumptions.

---

## Implementation Waves Timeline

```mermaid
timeline
    title Aipo Language Engineering Trajectory
    Wave 0 (MVP) : S1 to S3 (Lexer, Parser, AST)
                 : S4 to S6 (HIR, Sema, IR, Bytecode)
                 : S7 to S11 (VM, Stdlib, CLI, Conformance)
    Wave 1 (Contracts) : Construction Hooks (init)
                       : Invariants with Rollback (attempt)
                       : Structural Interfaces & Local Functions
    Wave 2 (JS Parity) : aipo-js Emitter & ES2022 Shim
                       : Differential Conformance VM vs JS
                       : Fuzzing & cargo deny Audit
    Wave 3 (Async) : Rich Types (Set, Sequence, Bytes)
                   : async fn & sequential await do
                   : Deterministic Scheduler with Virtual Time
    Wave 4 (Host ABI) : aipo-host Crate with Capability Trees
                      : Generational Handles Anti Use-After-Free
                      : Poppy Adapter & Headless Simulation
    Wave 5 (Packages) : aipo.toml Manifest & Canonical Lockfile
                      : SHA-Pinned GitHub Dependencies
                      : SHA-256 Verifiable Local Cache
    Wave 6 (v0.1.0) : Product Boundary Scoping v0.1.0 (ADP-008)
                    : Synchronous Versioned C ABI (ADP-009)
                    : Rust, C, and JS Thin Proofs (ADP-010)
```

---

## Implementation Waves Summary

<div class="wave-grid">

<div class="wave-card">
<span class="milestone-badge">Completed</span>
<h4>Wave 0 — Language MVP</h4>
<p>Implementation of 11 fundamental vertical slices: UTF-8 Lexer, resilient Parser, HIR, semantic analysis, Core IR, Bytecode, stack/register VM, minimal Stdlib, Code Formatter, unified CLI, and initial conformance suite.</p>
<p><a href="/en/trajectory/wave-0-mvp">Read Wave 0 documentation →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Completed</span>
<h4>Wave 1 — Contracts & Conformance</h4>
<p>Language MVP hardening with <code>init()</code> hooks, <code>invariant()</code> integrity enforcement on stable mutation boundaries with automated rollback via transactional journal, and structural interface conformance.</p>
<p><a href="/en/trajectory/wave-1-contracts">Read Wave 1 documentation →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Completed</span>
<h4>Wave 2 — JS Parity & Quality</h4>
<p>Modern JavaScript (ES2022) code generation via <code>aipo-js</code> with modular runtime shim, differential VM↔JS conformance testing, property-based testing, continuous fuzzing, and strict dependency audits with <code>cargo deny</code>.</p>
<p><a href="/en/trajectory/wave-2-js-parity">Read Wave 2 documentation →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Completed</span>
<h4>Wave 3 — Rich Types & Async</h4>
<p>Introduction of insertion-ordered <code>Set</code>, lazy <code>Sequence</code>, binary <code>Bytes</code> packaging, native <code>async fn</code> and <code>await do</code> syntax, async combinators, and a cooperative scheduler with virtual time.</p>
<p><a href="/en/trajectory/wave-3-async">Read Wave 3 documentation →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Completed</span>
<h4>Wave 4 — Host ABI & Poppy Engine</h4>
<p>Secure Host ABI design (<code>aipo-host</code>) featuring deny-by-default capabilities, generational handles resistant to use-after-free, scope escape barriers, and headless integration with the Poppy simulation engine.</p>
<p><a href="/en/trajectory/wave-4-host-poppy">Read Wave 4 documentation →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Completed</span>
<h4>Wave 5 — Hermetic Packages</h4>
<p>Hermetic package distribution and dependency resolution: <code>namespace.package</code> coordinates, commit SHA-pinned GitHub dependencies with hash verification, atomic local cache, and 100% offline replay.</p>
<p><a href="/en/trajectory/wave-5-packages">Read Wave 5 documentation →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">In Progress</span>
<h4>Wave 6 — Release v0.1.0</h4>
<p>Formal product boundary definition (ADP-008): dedicated focus on the standalone language and toolchain, minimal test runner, versioned synchronous C ABI (ADP-009), and thin interop proofs in Rust, C, and JavaScript (ADP-010).</p>
<p><a href="/en/trajectory/wave-6-release-v010">Read Wave 6 documentation →</a></p>
</div>

</div>
