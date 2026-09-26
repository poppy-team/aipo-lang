# Security & Sandboxing

Aipo security architecture is anchored in the principle of **least privilege** and proactive runtime vulnerability mitigation.

---

## Security Principles

1. **Zero Hardcoded Secrets**: Source code, test fixtures, and package manifests never store credentials, API keys, or access tokens.
2. **Deny by Default**: No host system capability (filesystem, environment variables, system clock, network sockets) is accessible to scripts without explicit permission granted by the host application.
3. **Memory Isolation Against Use-After-Free**: Host resources exposed to Aipo scripts utilize generational handles with atomic slot retirement.
4. **Denial of Service (DoS) Hardening**:
   - Maximum recursion depth ceiling in the parser (256 levels) preventing call stack exhaustion.
   - Optional instruction step budgeting (*gas budget*) preventing infinite execution loops in untrusted scripts.
   - Varint LEB128 maximum byte length limits mitigating integer overflow attacks.
5. **Supply Chain Hermeticity**:
   - Remote package dependencies require explicit 40-character commit SHAs in `aipo.toml`.
   - Automatic cryptographic verification against SHA-256 tree digests stored in local cache.
   - Production builds execute 100% offline, eliminating dynamic runtime dependency poisoning.
