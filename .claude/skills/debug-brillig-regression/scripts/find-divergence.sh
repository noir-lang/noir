#!/usr/bin/env bash
# Find where SSA passes first diverge between two branches.
# Usage: find-divergence.sh <base_passes_dir> <current_passes_dir>
#
# Compares passes by name (ignoring numeric prefix) and reports which
# passes differ, which are missing, and which is the first divergence.

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <base_passes_dir> <current_passes_dir>" >&2
    exit 1
fi

base_dir="$1"
current_dir="$2"
first_diff=""

for base_file in "$base_dir"/*.ssa; do
    [[ -f "$base_file" ]] || continue
    base_name=$(basename "$base_file" | sed 's/^[0-9]*_//')
    current_file=$(ls "$current_dir"/*"$base_name" 2>/dev/null | head -1)

    if [[ -n "$current_file" ]]; then
        if ! diff -q "$base_file" "$current_file" > /dev/null 2>&1; then
            echo "DIFFERS: $base_name"
            if [[ -z "$first_diff" ]]; then
                first_diff="$base_name"
            fi
        fi
    else
        echo "MISSING from current: $base_name"
    fi
done

# Check for passes only in current
for current_file in "$current_dir"/*.ssa; do
    [[ -f "$current_file" ]] || continue
    current_name=$(basename "$current_file" | sed 's/^[0-9]*_//')
    base_file=$(ls "$base_dir"/*"$current_name" 2>/dev/null | head -1)
    if [[ -z "$base_file" ]]; then
        echo "NEW in current: $current_name"
    fi
done

echo ""
if [[ -n "$first_diff" ]]; then
    echo "First divergence: $first_diff"
else
    echo "No divergence found — all shared passes are identical."
fi
