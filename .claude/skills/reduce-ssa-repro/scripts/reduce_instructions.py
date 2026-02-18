#!/usr/bin/env python3
"""Remove unused instructions from SSA while preserving a crash.

Usage: python3 reduce_instructions.py [--ssa-binary PATH] [--passes PASS1 PASS2 ...]

Reads `input.ssa` from the current directory, removes instructions whose result
values are not referenced elsewhere, and writes the reduced version back.

The crash condition is: the SSA parses successfully AND applying the given passes
triggers an "Unmapped value" panic (or other specified error pattern).
"""
import argparse
import re
import subprocess
import tempfile

def parse_args():
    parser = argparse.ArgumentParser(description="Remove unused SSA instructions while preserving a crash.")
    parser.add_argument("--ssa-binary", default="../target/debug/noir-ssa", help="Path to noir-ssa binary")
    parser.add_argument("--input", default="input.ssa", help="Input SSA file")
    parser.add_argument("--passes", nargs="+", required=True,
                        help="SSA passes to apply (as pass names), e.g. --passes Unrolling 'Inlining Brillig Calls'")
    parser.add_argument("--error-pattern", required=True,
                        help="Pattern to look for in stderr to confirm the crash")
    return parser.parse_args()

def build_pass_args(pass_names):
    args = []
    for name in pass_names:
        args.extend(["--ssa-pass", name])
    return args

def crashes(ssa_text, noir_ssa, pass_args, error_pattern):
    """Check if the SSA crashes when the pipeline is applied."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.ssa', delete=False) as f:
        f.write(ssa_text)
        f.flush()
        # First validate it parses
        r = subprocess.run(
            [noir_ssa, "check", "--source-path", f.name],
            capture_output=True, text=True, timeout=30
        )
        if r.returncode != 0:
            return False  # Doesn't parse, not useful
        # Now try the pipeline
        r = subprocess.run(
            [noir_ssa, "transform", "--source-path", f.name, "-o", "/dev/null"] + pass_args,
            capture_output=True, text=True, timeout=60
        )
        return r.returncode != 0 and error_pattern in r.stderr

def try_remove_instruction(ssa_text, line_idx):
    """Try removing a single instruction line."""
    lines = ssa_text.split('\n')
    line = lines[line_idx].strip()
    # Don't remove block headers, terminators, or structural lines
    if (not line or line.startswith('b') or line.startswith('}') or
        line.startswith('jmp') or line.startswith('jmpif') or
        line.startswith('return') or line.startswith('//') or
        line.startswith('g') or 'fn ' in line or line == '{'):
        return None

    # Get the variable this instruction defines
    m = re.match(r'(v\d+)\s*=', line)
    if m:
        var = m.group(1)
        # Check if this variable is used elsewhere
        rest = '\n'.join(lines[:line_idx] + lines[line_idx+1:])
        if re.search(rf'\b{var}\b', rest):
            # Variable is used elsewhere, can't just remove
            return None

    new_lines = lines[:line_idx] + lines[line_idx+1:]
    return '\n'.join(new_lines)

def main():
    args = parse_args()
    noir_ssa = args.ssa_binary

    pass_args = build_pass_args(args.passes)

    with open(args.input) as f:
        current = f.read()

    print(f"Original: {len(current.splitlines())} lines, crashes: {crashes(current, noir_ssa, pass_args, args.error_pattern)}")

    total_removed = 0

    while True:
        improved = False
        lines = current.split('\n')

        for i in range(len(lines) - 1, -1, -1):
            candidate = try_remove_instruction(current, i)
            if candidate is None:
                continue
            if crashes(candidate, noir_ssa, pass_args, args.error_pattern):
                current = candidate
                total_removed += 1
                if total_removed % 10 == 0:
                    print(f"  Removed {total_removed} instructions so far...")
                improved = True
                break

        if not improved:
            break

    with open(args.input, "w") as f:
        f.write(current)

    print(f"\nRemoved {total_removed} instructions total")
    print(f"Final: {len(current.splitlines())} lines")
    print(f"Crash still reproduces: {crashes(current, noir_ssa, pass_args, args.error_pattern)}")

if __name__ == "__main__":
    main()
