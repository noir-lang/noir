#!/usr/bin/env python3
"""Remove unused instructions from SSA while preserving a crash.

Usage: python3 reduce_instructions.py [--ssa-binary PATH] [--passes PASS1 PASS2 ...]

Reads `input.ssa` from the current directory, removes instructions whose result
values are not referenced elsewhere, and writes the reduced version back.

The crash condition is: the SSA parses successfully AND applying the given passes
triggers an "Unmapped value" panic (or other specified error pattern).
"""
from ssa_reduce_common import parse_args, build_pass_args, crashes, remove_unused_instructions

def main():
    args = parse_args("Remove unused SSA instructions while preserving a crash.")
    noir_ssa = args.ssa_binary
    pass_args = build_pass_args(args.passes)

    with open(args.input) as f:
        current = f.read()

    print(f"Original: {len(current.splitlines())} lines, crashes: {crashes(current, noir_ssa, pass_args, args.error_pattern)}")

    current, total_removed = remove_unused_instructions(
        current, noir_ssa, pass_args, args.error_pattern, progress_interval=10
    )

    with open(args.input, "w") as f:
        f.write(current)

    print(f"\nRemoved {total_removed} instructions total")
    print(f"Final: {len(current.splitlines())} lines")
    print(f"Crash still reproduces: {crashes(current, noir_ssa, pass_args, args.error_pattern)}")

if __name__ == "__main__":
    main()
