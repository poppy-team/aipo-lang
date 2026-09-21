# aipo-host

`aipo-host` is the host ABI of Aipo. It exists so the compiler never learns what a host *is*: a host describes its surface as data, reaches its services only through a deny-by-default capability set, and hands scripts plain values plus generational references to the objects it owns.

## Architecture & Guarantees

- **Aipo Host Schema (`HostSchema`)**: the host surface as data — modules, types, host values, handles, functions, params, return contracts, mutability, async, docs, capabilities and deprecation. Consumed by the compiler, the LSP and code agents without naming an engine. `validate` rejects an internally inconsistent surface (duplicate names, a subject that is not a parameter, a malformed capability) and reports every problem at once.
- **Deny-by-default capabilities (`CapabilitySet`)**: capabilities are dotted paths and the path is the tree, so granting `clock` grants `clock.wall` and `clock.monotonic`. A declared set is an upper bound; `narrow` only ever removes grants. `require` is the single door a host service goes through, so a denial is always the same fault with the same code.
- **Generational handles (`HandleTable`)**: a handle is a slot index plus a generation, and a handle is only dereferenced through the table that minted it. Releasing a slot bumps its generation, so a handle from before the release resolves to stale rather than to whatever reuses the slot. When a generation space is exhausted the slot is retired instead of wrapping.
- **Host values (`HostValue`)**: a closed enum of plain data plus a `Handle`. Constructors enforce the language's numeric invariants at the boundary — `Int` within ±(2^53−1), `Float` finite-only — so a host cannot smuggle an out-of-range value into a script.
- **Faults, not panics**: a denied capability, a stale handle and a scoped binding that escapes each map to a stable diagnostic code (`AIPO_RT_CAPABILITY_DENIED`, `AIPO_RT_STALE_HANDLE`, `AIPO_RT_SCOPE_ESCAPE`), and a rejected host value is a contract fault. No Rust reference or lifetime ever crosses the boundary.
- **Safety**: `#![forbid(unsafe_code)]`; the crate depends only on `aipo-diagnostics` and serde.

## Status

Slice `P03-G01` (Wave 4). The VM-side adapter — converting `HostValue` into a language value, turning `HostFault` into a runtime fault, enforcing scoped escape at the heap-publication points and gating `time.now`/`time.monotonic` behind the `clock` capability — is the remaining work of the slice; this crate is the contract that adapter is written against. `aipo-poppy` (slice `P03-G02`) builds the ECS scopes, command buffer and behaviors/events on top of this ABI.

## Testing

`cargo test -p aipo-host` covers each guarantee directly: capability subsumption and narrowing, the deny-by-default path through `require`, stale-handle resolution across slot reuse, generation exhaustion, value-range rejection, and AHS validation of well-formed and malformed surfaces.
