#!/usr/bin/env bash
# Bisect SSA passes to find where a failure is introduced.
# Usage: bisect-ssa.sh <input_toml> [ssa_dir] [noir_ssa_binary]
#
# Example: bisect-ssa.sh 'v0 = "-92"'
#          bisect-ssa.sh 'v0 = "42"; v1 = "100"' ssa_passes /path/to/noir-ssa

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <input_toml> [ssa_dir] [noir_ssa_binary]" >&2
    echo "" >&2
    echo "Arguments:" >&2
    echo "  input_toml      Input values in TOML format (use ; for multiple lines)" >&2
    echo "  ssa_dir         Directory containing .ssa files (default: ssa_passes)" >&2
    echo "  noir_ssa_binary Path to noir-ssa binary (default: noir-ssa in PATH)" >&2
    exit 1
fi

input_toml="$1"
ssa_dir="${2:-ssa_passes}"
noir_ssa="${3:-noir-ssa}"

if [[ ! -d "$ssa_dir" ]]; then
    echo "Error: Directory '$ssa_dir' not found" >&2
    exit 1
fi

if ! command -v "$noir_ssa" &>/dev/null; then
    echo "Error: noir-ssa binary not found at '$noir_ssa'" >&2
    echo "Build it with: cargo build --release -p noir_ssa_cli" >&2
    exit 1
fi

last_result=""
first_failure=""

for f in "$ssa_dir"/*.ssa; do
    [[ -f "$f" ]] || continue

    filename=$(basename "$f")

    if result=$("$noir_ssa" interpret --source-path "$f" --input-toml "$input_toml" 2>&1); then
        is_error=false
    else
        is_error=true
    fi

    output=$(echo "$result" | grep -E "^(Ok|Err)" | head -1)
    echo "$filename: $output"

    # Track first failure based on output starting with "Err"
    if [[ "$output" == Err* && -z "$first_failure" ]]; then
        first_failure="$filename"
    fi

    last_result="$output"
done

echo ""
if [[ -n "$first_failure" ]]; then
    echo "First failure: $first_failure"
else
    echo "All passes succeeded"
fi
