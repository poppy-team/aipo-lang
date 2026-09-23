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

## Command surface

```text
aipo run <path> [--message-format=<human|jsonl>]
aipo check <path> [--message-format=<human|jsonl>]
aipo build <path> [--out <dir>] [--message-format=<human|jsonl>]
aipo disasm <path> [--message-format=<human|jsonl>]
aipo fmt <paths...> [--check]
aipo --version
aipo --help
```

## Machine output

When `--message-format=jsonl` is specified, all compiler errors, warnings, and runtime faults are emitted to standard error as newline-delimited JSON matching the diagnostic catalog schema:

```json
{"code":"AIPO_SEM_UNKNOWN_NAME","severity":"error","message":"unknown variable 'y'","primary_span":{"file":"main.aipo","start":11,"end":16,"line":2,"column":1},"notes":[],"suggestions":[]}
```

## Exit codes

- `0`: Success (program ran to completion, check found no errors, or format succeeded).
- `1`: Language failure (parse error, semantic error, uncaught runtime fault, or check failed).
- `2`: CLI usage error (invalid command-line arguments, missing file, unrecognized flag).

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

