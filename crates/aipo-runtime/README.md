# aipo-runtime

`aipo-runtime` provides shared runtime coordination services for the Aipo VM: module registry, dependency graph analysis, cycle detection, deterministic initialization ordering, and native function catalogs.

## Architecture & Guarantees
- **Module Graph (`ModuleGraph`)**: Maintains registered module records and calculates deterministic topological initialization order.
- **Topological Initialization Order**: Ensures imported dependencies initialize before importer modules, with lexicographical canonical path tie-breaking.
- **Cycle Detection**: Detects circular imports (`AIPO_SEM_IMPORT_CYCLE`) at load/init time.
- **Native Registry (`NativeRegistry`)**: Catalogs native built-in functions for the Prelude and standard modules.
- **Safety**: `#![forbid(unsafe_code)]`. Zero unhandled panics.
