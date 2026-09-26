# Historical Slice Register

Every vertical slice and milestone in Aipo is completed only after automated evidence passes and a formal certification report is recorded in `docs/evidence/`.

---

## Consolidated Milestone History

### Phase P00: MVP Foundation & Contracts Closure
- **P00-G01 through P00-G08**: Workspace setup, Lexer, Parser, HIR, SEMA, Core IR, Bytecode, VM, and Diagnostic Fault Model.
- **P00-G09**: `aipo-runtime` crate and first standard library (*MVP subset*).
- **P00-G10**: End-to-end pipeline integration (Syntax $\rightarrow$ IR $\rightarrow$ Bytecode $\rightarrow$ VM $\rightarrow$ CLI).
- **P00-G11**: Unified CLI utility and canonical code formatter.
- **P00-G12**: MVP quality and conformance verification gate.
- **P00-G13**: `init()` construction hooks and native `Bytes` type.
- **P00-G14**: Invariant enforcement on stable mutations and journaled rollback inside `attempt`.
- **P00-G15**: Static `interface` contracts, structural dynamic dispatch conformance, and Unicode NFC normalization.
- **P00-G16**: Wave 1 exit review with 100% green gauntlet (13/13, 11/11, 3/3, 146/146).

### Phase P01: JavaScript Parity & Deep Quality Gauntlet
- **P01-G01**: `aipo-js` compiler, versioned runtime shim, Source Maps V3, and 100% differential parity VM ↔ Node.js.
- **P01-G02**: Deep fuzzing suite (libFuzzer), property-based testing (proptest), and supply-chain auditing (`cargo deny`).

### Phase P02: Wave 3 — Rich Types & Async
- **P02-G01**: Insertion-ordered `Set`, lazy `Sequence`, and endian-aware `Bytes` binary packing.
- **P02-G02**: Async task combinators (`task.*`) and cooperative scheduler with deterministic virtual time.
- **P02-G03**: `async fn` syntax, sequential `await do ... end`, semantic diagnostics, and task dependency cycle detection.

### Phase P03: Wave 4 — Host ABI & Poppy Engine
- **P03-G01**: `aipo-host` crate, Host Schema (AHS), deny-by-default capabilities, generational handles, and scope escape prevention.
- **P03-G02**: `aipo-poppy` crate, `poppy` module, transactional command buffers, and seeded deterministic headless simulation.

### Phase P04: Wave 5 — Hermetic Package Manager
- **P04-G01 through P04-G11**: Hermetic packaging system, `aipo.toml` manifest, deterministic lockfiles, commit SHA-pinned GitHub dependencies, content-addressable local cache with SHA-256 verification, and cache maintenance tools (`verify`, `prune`).
