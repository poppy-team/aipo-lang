#!/usr/bin/env bash
# Run paired A/B benchmark binaries with a pinned CPU and alternating order.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/perf/paired.sh --baseline PATH --candidate PATH [options]

Required:
  --baseline PATH     Baseline aipo-bench binary
  --candidate PATH    Candidate aipo-bench binary

Options:
  --workloads LIST    Comma-separated workload ids (default: arithmetic,collections,fields,recursion)
  --languages LIST    Comma-separated language ids (default: aipo-vm)
  --rounds N          Samples per invocation, 1..31 (default: 15)
  --pairs N           Paired invocations, 1..9 (default: 3)
  --cpu N             CPU id passed to taskset (default: 0)
  --output-dir PATH   Report directory (default: target/paired)
  --allow-unpinned    Continue when taskset is unavailable
  -h, --help          Show this help

The baseline and candidate are run alternately; odd pairs run baseline first,
even pairs run candidate first. The script never changes Git state.
EOF
}

baseline=""
candidate=""
workloads="arithmetic,collections,fields,recursion"
languages="aipo-vm"
rounds=15
pairs=3
cpu=0
output_dir="target/paired"
allow_unpinned=0

while (($# > 0)); do
  case "$1" in
    --baseline)
      baseline="${2:-}"
      shift 2
      ;;
    --candidate)
      candidate="${2:-}"
      shift 2
      ;;
    --workloads)
      workloads="${2:-}"
      shift 2
      ;;
    --languages)
      languages="${2:-}"
      shift 2
      ;;
    --rounds)
      rounds="${2:-}"
      shift 2
      ;;
    --pairs)
      pairs="${2:-}"
      shift 2
      ;;
    --cpu)
      cpu="${2:-}"
      shift 2
      ;;
    --output-dir)
      output_dir="${2:-}"
      shift 2
      ;;
    --allow-unpinned)
      allow_unpinned=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

[[ -n "$baseline" && -n "$candidate" ]] || {
  echo "error: --baseline and --candidate are required" >&2
  usage >&2
  exit 2
}
[[ "$rounds" =~ ^[0-9]+$ && "$rounds" -ge 1 && "$rounds" -le 31 ]] || {
  echo "error: --rounds must be between 1 and 31" >&2
  exit 2
}
[[ "$pairs" =~ ^[0-9]+$ && "$pairs" -ge 1 && "$pairs" -le 9 ]] || {
  echo "error: --pairs must be between 1 and 9" >&2
  exit 2
}
[[ "$cpu" =~ ^[0-9]+$ ]] || {
  echo "error: --cpu must be a non-negative integer" >&2
  exit 2
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
caller_dir="$PWD"
resolve_binary() {
  local path="$1"
  if [[ "$path" != /* ]]; then
    path="$caller_dir/$path"
  fi
  [[ -x "$path" ]] || {
    echo "error: benchmark binary is not executable: $path" >&2
    exit 2
  }
  printf '%s\n' "$path"
}
baseline="$(resolve_binary "$baseline")"
candidate="$(resolve_binary "$candidate")"

if ! command -v taskset >/dev/null 2>&1; then
  if ((allow_unpinned == 0)); then
    echo "error: taskset is required; use --allow-unpinned to run without CPU pinning" >&2
    exit 2
  fi
  pin=()
else
  pin=(taskset -c "$cpu")
fi

mkdir -p "$repo_root/$output_dir"
output_dir="$repo_root/$output_dir"
cd "$repo_root"

{
  printf 'created_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'baseline=%s\n' "$baseline"
  printf 'candidate=%s\n' "$candidate"
  printf 'workloads=%s\n' "$workloads"
  printf 'languages=%s\n' "$languages"
  printf 'rounds=%s\n' "$rounds"
  printf 'pairs=%s\n' "$pairs"
  printf 'cpu=%s\n' "$cpu"
  printf 'taskset=%s\n' "$([[ ${#pin[@]} -gt 0 ]] && echo enabled || echo disabled)"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$baseline" "$candidate"
  fi
} > "$output_dir/manifest.txt"

run_one() {
  local label="$1"
  local binary="$2"
  local pair="$3"
  local report="$output_dir/${label}-${pair}.json"
  printf '\n[%s] %s (pair %s/%s)\n' "$label" "$binary" "$pair" "$pairs"
  "${pin[@]}" "$binary" \
    --compare \
    --compare-languages "$languages" \
    --compare-workloads "$workloads" \
    --compare-runs "$rounds" \
    --compare-json "$report"
}

for ((pair = 1; pair <= pairs; pair++)); do
  if ((pair % 2 == 1)); then
    run_one baseline "$baseline" "$pair"
    run_one candidate "$candidate" "$pair"
  else
    run_one candidate "$candidate" "$pair"
    run_one baseline "$baseline" "$pair"
  fi
done

printf '\nPaired reports and manifest:\n  %s\n' "$output_dir"
