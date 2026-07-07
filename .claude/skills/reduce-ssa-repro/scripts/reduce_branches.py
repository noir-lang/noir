#!/usr/bin/env python3
"""Reduce SSA by converting conditional branches to unconditional jumps.

Usage: python3 reduce_branches.py [--ssa-binary PATH] [--passes PASS1 PASS2 ...]

Reads `input.ssa` from the current directory. For each `jmpif` instruction,
tries replacing it with an unconditional `jmp` to either the then or else target.
After each successful branch elimination, re-runs instruction removal to clean up
newly-unused code. Writes the reduced version back to `input.ssa`.
"""
import re
from ssa_reduce_common import parse_args, build_pass_args, crashes, remove_unused_instructions

def try_redirect_jmpif(text, line_idx, to_then):
    """Replace a jmpif with an unconditional jmp to one of its targets."""
    lines = text.split('\n')
    line = lines[line_idx]
    m = re.search(r'jmpif\s+(\S+)\s+then:\s+(b\d+),\s+else:\s+(b\d+)', line)
    if not m:
        return None
    target = m.group(2) if to_then else m.group(3)
    lines[line_idx] = f'    jmp {target}()'
    return '\n'.join(lines)

def main():
    args = parse_args("Reduce SSA branches while preserving a crash.")
    noir_ssa = args.ssa_binary
    pass_args = build_pass_args(args.passes)

    with open(args.input) as f:
        current = f.read()

    print(f"Start: {len(current.splitlines())} lines, crashes: {crashes(current, noir_ssa, pass_args, args.error_pattern)}")

    round_num = 0
    overall_improved = True
    while overall_improved:
        overall_improved = False
        lines = current.split('\n')

        # Find all jmpif lines
        jmpif_indices = []
        for i, line in enumerate(lines):
            if re.search(r'jmpif\s+\S+\s+then:', line):
                jmpif_indices.append(i)

        for line_idx in jmpif_indices:
            # Try redirecting to else first (often eliminates loop bodies)
            for to_then in [False, True]:
                candidate = try_redirect_jmpif(current, line_idx, to_then)
                if candidate and crashes(candidate, noir_ssa, pass_args, args.error_pattern):
                    direction = "then" if to_then else "else"
                    m = re.search(r'jmpif\s+\S+\s+then:\s+(b\d+),\s+else:\s+(b\d+)', lines[line_idx])
                    target = m.group(1) if to_then else m.group(2)
                    round_num += 1
                    print(f"  Round {round_num}: Redirected jmpif at line {line_idx} to {direction} ({target})")
                    current = candidate

                    # Clean up newly unused instructions
                    current, removed = remove_unused_instructions(current, noir_ssa, pass_args, args.error_pattern)
                    if removed:
                        print(f"    Cleaned up {removed} unused instructions")

                    overall_improved = True
                    break
            if overall_improved:
                break

    with open(args.input, "w") as f:
        f.write(current)

    print(f"\nFinal: {len(current.splitlines())} lines")
    print(f"Crash still reproduces: {crashes(current, noir_ssa, pass_args, args.error_pattern)}")

if __name__ == "__main__":
    main()
