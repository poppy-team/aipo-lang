# Aipo — Installation and Lifecycle

**Status:** normative
**Scope:** installation paths, ownership, rollback strategy, uninstall safety
**Blocking question:** What happens after interrupted update?

## Installation paths

- Single binary executable: `aipo` (placed in `~/.local/bin` or `/usr/local/bin`).
- Standard library root: embedded in executable or located in `~/.local/share/aipo/stdlib/`.

## Ownership and permissions

- Binary belongs to user or root depending on install scope.
- No background daemon or privileged background service is installed.

## Rollback strategy

- Standalone binaries are installed via atomic move (`mv aipo.new aipo`).
- If an update is interrupted, the previous working binary remains intact because the atomic replacement has not occurred.

## Uninstall safety

- Removing the `aipo` binary and the optional cache directory `~/.cache/aipo/` cleanly uninstalls the toolchain without side effects on system libraries.
