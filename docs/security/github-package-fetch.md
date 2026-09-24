# Public GitHub package fetch security

**Status:** implemented for the opt-in `github-http` feature
**Scope:** `aipo package fetch-github` and its explicit recursive GitHub graph
**Network policy:** public GET only; no credentials; no automatic downloads

## Trust boundary

The GitHub response is untrusted input. The trusted boundary is the local `aipo-package` parser,
cache verifier, graph resolver and output writer. A response becomes a package only after its
manifest, entry path, content digest, source revision, coordinate, version and dependency edges pass
validation. A fetched graph is written only after the complete root-plus-transitive graph resolves.

The cache directory is caller-owned. The command never chooses a global cache, never reads tokens,
and never writes outside the explicit `--cache` and `--out` directories.

## Required controls

- The feature is disabled in the default CLI build.
- The source accepts only `owner/repository`, an exact lowercase 40-hex commit and a safe relative subpath.
- Requests use HTTPS, a fixed GitHub raw host, a bounded timeout and a 4 MiB response limit.
- Requests do not send `Authorization` and the transport disables redirects.
- Cache paths use a SHA-256 key instead of repository text; symlinks are rejected.
- Cache writes use a private temporary directory and atomic rename.
- Cached manifest and entry bytes are parsed and checked against the stored SHA-256 digest.
- Corrupt cache entries fail closed; the command does not silently repair or overwrite them.
- The output directory must be empty. Existing files are never overwritten.
- Every recursively fetched source is cached and digest-verified independently.
- A remote graph is bounded to 256 packages per fetch operation.
- Remote artifacts may declare only pinned GitHub dependencies; local `path` dependencies inside a
  remote artifact fail closed because no caller-owned filesystem root is available.
- Package lifecycle code is never executed.

## Operational ownership

The user chooses the cache root and owns its retention and permissions. The cache has no automatic
eviction in this slice. Delete it explicitly when removal is required. Use a dedicated directory
when the host workspace is shared.

## Non-goals

This slice does not support private repositories, tokens, OAuth, authenticated requests, persistent
cache eviction, signatures, provenance attestations, registry publication or automatic dependency
resolution by `run`, `check`, `build`, `disasm`, `package lock` or `package audit`. The explicit
`fetch-github` command is the only network-triggering operation.
