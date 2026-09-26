#!/usr/bin/env python3
"""A/B two aipo-bench binaries by child CPU time instead of wall-clock.

Why: this host is shared and the wall-clock timer is dominated by other tenants. Under
contention, wall-clock inflates by an arbitrary factor while CPU time (user+sys) stays
proportional to the work actually done, because descheduled time is not charged to us.
This measures `resource.getrusage(RUSAGE_CHILDREN)` deltas around each run and reports
the minimum per side, which is the robust estimator when noise is additive.

Usage: scripts/perf/cpu_ab.py BASELINE CANDIDATE [--workloads a,b] [--rounds N] [--reps N]
"""
from __future__ import annotations

import argparse
import resource
import statistics
import subprocess
import sys


def child_cpu() -> float:
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    return usage.ru_utime + usage.ru_stime


def run(binary: str, workloads: str, rounds: int) -> float:
    before = child_cpu()
    proc = subprocess.run(
        [
            binary,
            "--compare",
            "--compare-languages",
            "aipo-vm",
            "--compare-workloads",
            workloads,
            "--compare-runs",
            str(rounds),
        ],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise SystemExit(f"{binary} failed: {proc.stderr[-500:]}")
    return child_cpu() - before


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline")
    parser.add_argument("candidate")
    parser.add_argument("--workloads", default="arithmetic")
    parser.add_argument("--rounds", type=int, default=10)
    parser.add_argument("--reps", type=int, default=6)
    args = parser.parse_args()

    samples: dict[str, list[float]] = {"baseline": [], "candidate": []}
    # Alternate which side goes first so any monotone drift cancels.
    for index in range(args.reps):
        order = ("baseline", "candidate") if index % 2 == 0 else ("candidate", "baseline")
        for side in order:
            binary = args.baseline if side == "baseline" else args.candidate
            samples[side].append(run(binary, args.workloads, args.rounds))

    print(f"workloads={args.workloads} rounds={args.rounds} reps={args.reps} (child CPU seconds)")
    for side in ("baseline", "candidate"):
        values = samples[side]
        print(
            f"  {side:9s} min={min(values):7.3f} median={statistics.median(values):7.3f} "
            f"all={[round(v, 2) for v in values]}"
        )
    base_min, cand_min = min(samples["baseline"]), min(samples["candidate"])
    base_med = statistics.median(samples["baseline"])
    cand_med = statistics.median(samples["candidate"])
    print(
        f"  delta(min)    = {100 * (cand_min - base_min) / base_min:+6.2f}%\n"
        f"  delta(median) = {100 * (cand_med - base_med) / base_med:+6.2f}%"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
