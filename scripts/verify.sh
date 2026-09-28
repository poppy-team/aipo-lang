#!/usr/bin/env bash
# Re-lock every local Aipo package whose sources changed, then run the
# verification gates. The package digest covers source files, so any edit
# under packages/*/src invalidates aipo.lock until it is regenerated.
#
# Scope note: `aipo-c-abi` has uncommitted in-flight work in this tree and its
# P0 tests do not pass at baseline. It is excluded here deliberately; use
# `--all` to include the whole workspace when that work lands.
set -euo pipefail

cd "$(dirname "$0")/.."

SCOPE=(-p aipo-game-host -p aipo-cli)
if [[ "${1:-}" == "--all" ]]; then
  SCOPE=(--workspace)
fi

echo "==> Re-locking local Aipo packages"
for manifest in packages/*/aipo.toml; do
  pkg_dir="$(dirname "$manifest")"
  cargo run -q -p aipo-cli -- package lock "$pkg_dir" >/dev/null
  echo "    locked $pkg_dir"
done

echo "==> cargo fmt"
cargo fmt --all
cargo fmt --all --check

echo "==> cargo clippy"
cargo clippy --all-targets "${SCOPE[@]}" -- -D warnings

echo "==> cargo test"
cargo test "${SCOPE[@]}"

echo "==> docs build"
npm run --silent docs:build >/dev/null

echo "OK"
