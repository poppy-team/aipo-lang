#!/usr/bin/env bash
# LLVM source-based coverage for the Aipo workspace (no extra tooling beyond
# the `llvm-tools` rustup component).
#
# Usage: docs/testing/coverage.sh [--report PATH]
# Output: per-crate line/region/function summary on stdout plus an optional
# full report file. Profraw files land in `target/coverage/` (git-ignored via
# `target/`).
set -euo pipefail
cd "$(dirname "$0")/../.."

LLVM_BIN="$HOME/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin"
export LLVM_PROFILE_FILE="$PWD/target/coverage/cov-%p-%m.profraw"
mkdir -p target/coverage
rm -f target/coverage/*.profraw target/coverage/cov.profdata

RUSTFLAGS="-C instrument-coverage" \
    cargo test --workspace --all-targets > target/coverage/test.log 2>&1 || {
    echo "tests failed under coverage; see target/coverage/test.log"
    exit 1
}

"$LLVM_BIN/llvm-profdata" merge -sparse target/coverage/cov-*.profraw \
    -o target/coverage/cov.profdata

BINARIES=()
while IFS= read -r binary; do
    BINARIES+=(-object "$binary")
done < <(find target/debug/deps -maxdepth 1 -type f -executable \
    -name "aipo*" -o -type f -executable -name "conformance*" \
    -o -type f -executable -name "differential*" \
    -o -type f -executable -name "golden*" \
    -o -type f -executable -name "graph_tests*" \
    -o -type f -executable -name "stdlib_tests*" \
    -o -type f -executable -name "data_and_errors*" \
    -o -type f -executable -name "pipeline_integration*" \
    -o -type f -executable -name "vm_core*" \
    -o -type f -executable -name "fuzz_smoke*" \
    -o -type f -executable -name "js_build*" \
    -o -type f -executable -name "properties*" \
    -o -type f -executable -name "generated*" \
    -o -type f -executable -name "metamorphic*" \
    -o -type f -executable -name "sourcemap*" \
    -o -type f -executable -name "hostile*" \
    -o -type f -executable -name "resource*" \
    -o -type f -executable -name "examples*" \
    -o -type f -executable -name "diagnostic_ui*" \
    -o -type f -executable -name "determinism*" \
    -o -type f -executable -name "fmt_equiv*" \
    -o -type f -executable -name "unicode_*" \
    -o -type f -executable -name "totality*" \
    -o -type f -executable -name "contracts_matrix*" \
    -o -type f -executable -name "value_properties*" \
    -o -type f -executable -name "suggestions*" \
    2>/dev/null)

REPORT="${1:-}"
if [ "${1:-}" = "--report" ]; then
    REPORT="$2"
fi
if [ -n "$REPORT" ]; then
    "$LLVM_BIN/llvm-cov" report \
        --instr-profile target/coverage/cov.profdata \
        "${BINARIES[@]}" \
        --show-functions > "$REPORT"
    echo "full report: $REPORT"
fi
"$LLVM_BIN/llvm-cov" report \
    --instr-profile target/coverage/cov.profdata \
    "${BINARIES[@]}" \
    --summary-only
