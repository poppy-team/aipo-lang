# Command-Line Interface Guide (`aipo`)

The `aipo` executable is the unified tool for execution, static checking, compilation, formatting, and package management.

---

## Primary Subcommands

### `aipo run`

Executes source code (`.aipo`) or precompiled bytecode (`.aibc`):

```bash
# Run source code directly
aipo run main.aipo

# Run compiled bytecode module
aipo run app.aibc

# Pass script arguments
aipo run script.aipo -- --flag value
```

### `aipo check`

Runs the complete frontend analysis pipeline (Lexer, Parser, HIR, and SEMA) without VM execution:

```bash
aipo check src/main.aipo
```

Reports syntax faults, unsatisfied interface contracts, unbound variables, or invalid reassignments to `fixed` fields.

### `aipo build`

Compiles `.aipo` files to supported targets:

```bash
# Compile to binary bytecode (.aibc)
aipo build src/main.aipo -o dist/main.aibc

# Compile to modern JavaScript (.js)
aipo build src/main.aipo -t js -o dist/bundle.js
```

### `aipo disasm`

Disassembles source files or `.aibc` binaries, displaying instruction mnemonics, constants, and mapped source coordinates:

```bash
aipo disasm src/main.aipo
```

### `aipo fmt`

Formats Aipo source files according to canonical style conventions:

```bash
# Format file in place
aipo fmt src/main.aipo

# Verify formatting (CI check)
aipo fmt --check src/
```

### `aipo package`

Hermetic package manager commands:

```bash
# Resolve and write deterministic aipo.lock
aipo package lock

# Download remote GitHub dependencies into local cache
aipo package fetch-github

# Verify cryptographic SHA-256 integrity of cached packages
aipo package cache verify .aipo/cache

# Prune unreferenced cache entries safely
aipo package cache prune .aipo/cache --lock aipo.lock --apply
```
