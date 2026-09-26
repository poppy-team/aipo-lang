# Host ABI & Sandboxing

The **Host ABI (`aipo-host`)** governs the sandboxing boundary and isolation model bridging Aipo scripts and host runtime environments (such as game engines, backend servers, or AI agents).

---

## Deny-by-Default Capabilities

By default, an Aipo script executes inside an hermetically isolated sandbox:
- Cannot access environment variables (`env`).
- Cannot read from or write to the filesystem (`fs`).
- Cannot query host system wall-clock time (`clock`).
- Cannot initiate outbound network connections.

For any resource to be accessible, the host application must explicitly grant granular permissions via the capability tree (`CapabilitySet`):

```rust
// Rust host example: granting clock and restricted read-only filesystem access
let mut caps = CapabilitySet::new();
caps.grant("clock");
caps.grant("fs.read");
```

If a script attempts to invoke a restricted capability without authorization, execution halts immediately with the deterministic fault `AIPO_RT_CAPABILITY_DENIED`.

---

## Generational Handles Anti Use-After-Free

For stateful host resources (such as simulation entities, file handles, or network sockets), Aipo utilizes a generational handle table (`HandleTable`):

- Every handle pairs an **index slot** with a **monotonic generation counter**.
- When a host resource is destroyed or recycled, its slot generation increments.
- Any subsequent attempt by a script to dereference a stale handle is intercepted with `AIPO_RT_STALE_HANDLE`, completely preventing dangling pointer dereferences and memory corruption.
