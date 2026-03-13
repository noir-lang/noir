#!/usr/bin/env python3
"""Diff two Brillig gates report JSON files and show regressions/improvements.

Usage: diff-reports.py <base_report.json> <current_report.json>

Compares unconstrained_functions_opcodes for each program and prints
any differences, flagging regressions with '!!!'.
"""

import json
import sys


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <base_report.json> <current_report.json>", file=sys.stderr)
        sys.exit(1)

    base_path, current_path = sys.argv[1], sys.argv[2]

    with open(base_path) as f:
        base = {p["package_name"]: p for p in json.load(f)["programs"]}
    with open(current_path) as f:
        curr = {p["package_name"]: p for p in json.load(f)["programs"]}

    any_diff = False
    for name in sorted(set(base) & set(curr)):
        b = base[name]["unconstrained_functions_opcodes"]
        c = curr[name]["unconstrained_functions_opcodes"]
        if c != b:
            any_diff = True
            delta = c - b
            pct = (delta / b * 100) if b else float("inf")
            marker = "!!!" if delta > 0 else ""
            print(f"{marker} {name}: {b} -> {c} ({delta:+d}, {pct:+.1f}%) {marker}")

    if not any_diff:
        print("No differences found.")


if __name__ == "__main__":
    main()
