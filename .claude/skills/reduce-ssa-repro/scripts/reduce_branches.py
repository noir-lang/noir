#!/usr/bin/env python3
"""Reduce SSA by converting conditional branches to unconditional jumps.

Usage: python3 reduce_branches.py [--ssa-binary PATH] [--passes PASS1 PASS2 ...]

Reads `input.ssa` from the current directory. For each `jmpif` instruction,
tries replacing it with an unconditional `jmp` to either the then or else target.
After each successful branch elimination, re-runs instruction removal to clean up
newly-unused code. Writes the reduced version back to `input.ssa`.
"""
import argparse
import re
import subprocess
import tempfile

def parse_args():
    parser = argparse.ArgumentParser(description="Reduce SSA branches while preserving a crash.")
    parser.add_argument("--ssa-binary", default="../target/release/noir-ssa", help="Path to noir-ssa binary")
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
        r = subprocess.run(
            [noir_ssa, "check", "--source-path", f.name],
            capture_output=True, text=True, timeout=30
        )
        if r.returncode != 0:
            return False
        r = subprocess.run(
            [noir_ssa, "transform", "--source-path", f.name, "-o", "/dev/null"] + pass_args,
            capture_output=True, text=True, timeout=60
        )
        return r.returncode != 0 and error_pattern in r.stderr

def try_remove_instruction(ssa_text, line_idx):
    """Try removing a single instruction line."""
    lines = ssa_text.split('\n')
    line = lines[line_idx].strip()
    if (not line or line.startswith('b') or line.startswith('}') or
        line.startswith('jmp') or line.startswith('jmpif') or
        line.startswith('return') or line.startswith('//') or
        line.startswith('g') or 'fn ' in line or line == '{'):
        return None

    m = re.match(r'(v\d+)\s*=', line)
    if m:
        var = m.group(1)
        rest = '\n'.join(lines[:line_idx] + lines[line_idx+1:])
        if re.search(rf'\b{var}\b', rest):
            return None

    new_lines = lines[:line_idx] + lines[line_idx+1:]
    return '\n'.join(new_lines)

def remove_unused_instructions(current, noir_ssa, pass_args, error_pattern):
    """Remove all unused instructions (one pass)."""
    total = 0
    while True:
        improved = False
        lines = current.split('\n')
        for i in range(len(lines) - 1, -1, -1):
            candidate = try_remove_instruction(current, i)
            if candidate is None:
                continue
            if crashes(candidate, noir_ssa, pass_args, error_pattern):
                current = candidate
                total += 1
                improved = True
                break
        if not improved:
            break
    return current, total

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
    args = parse_args()
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
