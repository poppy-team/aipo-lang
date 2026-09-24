# Aipo — CLI Reference

**Status:** normative
**Scope:** command surface, machine output, exit codes, and examples for Aipo CLI
**Blocking question:** Which commands are stable?

## Stable commands

The stable command surface comprises (`run`/`check`/`fmt` stable since Wave 1,
`build` stable since Wave 2, `disasm` stable since Wave 4):

- `aipo run <file.aipo>`: Compiles and executes an Aipo source file.
- `aipo run <file.aibc>`: Loads, verifies and executes a pre-compiled bytecode file.
- `aipo check <file.aipo>`: Runs the frontend and semantic analysis without bytecode execution, emitting diagnostics.
- `aipo build <file.aipo> [--out <dir>]`: Emits a JavaScript bundle (`app.js` + `aipo-runtime.js` + `app.js.map`, ESM with ECMA-426 source maps) to `<dir>` (`<parent>/dist` by default). Fails with the same diagnostic codes as `check` and emits no files on failure; runtime faults surface when `node` runs the bundle, with the same codes as `run`.
- `aipo disasm <file.aipo|file.aibc>`: Disassembles a source or bytecode file, printing a human-readable listing. Source files (`.aipo`) include line/column annotations; bytecode files (`.aibc`) show raw instruction offsets.
- `aipo fmt [files...]`: Formats source files idempotently according to canonical indentation rules.
- `aipo package lock <package-dir>`: Resolves a local package graph and writes `aipo.lock` in the package root.
- `aipo package audit <package-dir>`: Resolves a local package graph and verifies the existing `aipo.lock` without rewriting it.

## Local package entries

For `.aipo` inputs, `run`, `check`, `build`, and `disasm` look for the nearest `aipo.toml` in the
entry file's ancestor directories. When present, local `path` dependencies are resolved before
module linking. A qualified import such as `import acme.http` binds the local namespace `http`;
`import acme.http as h` binds `h`. The full coordinate remains the module identity.

Resolution for these source commands is local-only and executes no lifecycle scripts. A manifest
may also declare pinned GitHub dependencies, but `run`, `check`, `build` and `disasm` do not fetch
or consume them implicitly. A missing lockfile is allowed for these source commands. `aipo package
lock` and `aipo package audit` remain local package operations in this slice. Without an
`aipo.toml`, source files keep the legacy sibling-module lookup.

## Opt-in public GitHub fetch

The HTTP adapter is compiled only with the `github-http` feature. The default `aipo` binary remains
network-free. Enable it explicitly when fetching a public package:

```bash
cargo run -p aipo-cli --features github-http -- \\
  package fetch-github owner/repository 0123456789abcdef0123456789abcdef01234567 \\
  [subpath] --cache <cache-dir> --out <output-dir>
```

The command accepts only a public repository slug, an exact lowercase 40-hex commit, and a safe
relative subpath. A manifest dependency uses the same source fields and also requires an exact
SemVer version:

```toml
[dependencies]
"acme.http" = { version = "1.0.0", type = "github", repository = "acme/packages", revision = "0123456789abcdef0123456789abcdef01234567", subpath = "." }
```

The command sends public GET requests without `Authorization` or other credentials, rejects
redirects, applies a timeout and a 4 MiB response limit, and uses the explicit cache root. It
recursively fetches pinned GitHub dependencies, validates coordinates, versions, capabilities and
cycles, enforces a 256-package graph bound, and writes a complete graph lockfile in the output
snapshot. Each artifact is cached and
digest-verified independently. The output directory must be empty. A remote artifact cannot
declare a local `path` dependency in this slice.

Failures use `AIPO_PKG_FETCH` for transport/cache/artifact failures and `AIPO_PKG_RESOLUTION` for
graph failures. No cache or network action is attempted by `run`, `check`, `build`, `disasm`,
`package lock` or `package audit`; only the explicit `fetch-github` command enables the adapter.



```text
aipo run <path> [--message-format=<human|jsonl>]
aipo check <path> [--message-format=<human|jsonl>]
aipo build <path> [--out <dir>] [--message-format=<human|jsonl>]
aipo disasm <path> [--message-format=<human|jsonl>]
aipo fmt <paths...> [--check]
aipo package lock <package-dir>
aipo package audit <package-dir>
aipo --version
aipo --help
```

## Machine output

When `--message-format=jsonl` is specified, all compiler and package errors, warnings, and runtime faults are emitted to standard error as newline-delimited JSON matching the diagnostic catalog schema:

```json
{"code":"AIPO_SEM_UNKNOWN_NAME","severity":"error","message":"unknown variable 'y'","primary_span":{"file":"main.aipo","start":11,"end":16,"line":2,"column":1},"notes":[],"suggestions":[]}
```

## Exit codes

- `0`: Success (program ran to completion, check found no errors, or format succeeded).
- `1`: Language/package failure (parse error, semantic error, package resolution or stale-lock diagnostic, uncaught runtime fault, or check failed).
- `2`: CLI usage error (invalid command-line arguments, missing file, unrecognized flag, or filesystem failure).

## Examples

Run a script:
```bash
aipo run main.aipo
```

Run a pre-compiled bytecode file:
```bash
aipo run main.aibc
```

Check a script with machine-readable diagnostics:
```bash
aipo check main.aipo --message-format=jsonl
```

Disassemble a source file (with line annotations):
```bash
aipo disasm main.aipo
```

Disassemble a bytecode file:
```bash
aipo disasm main.aibc
```

Format files in-place:
```bash
aipo fmt src/**/*.aipo
```

Build a JavaScript bundle and run it with Node (requires Node >= 20):
```bash
aipo build main.aipo --out dist
node dist/app.js
```

Create and audit a local package lockfile:
```bash
aipo package lock .
aipo package audit .
```

