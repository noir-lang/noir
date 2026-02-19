"""Shared utilities for SSA reduction scripts."""
import argparse
import re
import subprocess
import tempfile


def parse_args(description):
    parser = argparse.ArgumentParser(description=description)
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


def remove_unused_instructions(current, noir_ssa, pass_args, error_pattern, progress_interval=0):
    """Remove all unused instructions iteratively.

    Args:
        progress_interval: If > 0, print progress every N removals.
    """
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
                if progress_interval > 0 and total % progress_interval == 0:
                    print(f"  Removed {total} instructions so far...")
                improved = True
                break
        if not improved:
            break
    return current, total
