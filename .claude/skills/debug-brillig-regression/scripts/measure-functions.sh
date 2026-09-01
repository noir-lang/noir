#!/usr/bin/env bash
# Measure per-function instruction counts and block parameters from final SSA.
# Usage: measure-functions.sh <base_passes_dir> <current_passes_dir>
#
# Compares the last SSA pass from each branch to show which functions
# grew or shrank, and total block parameter counts.

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <base_passes_dir> <current_passes_dir>" >&2
    exit 1
fi

base_dir="$1"
current_dir="$2"

echo "=== Instruction counts per function ==="
for branch_label in base current; do
    if [[ "$branch_label" == "base" ]]; then
        dir="$base_dir"
    else
        dir="$current_dir"
    fi

    echo "--- $branch_label ---"
    last=$(ls "$dir"/*.ssa | tail -1)
    awk '/^brillig.*fn / {name=$0; count=0} /=/ {count++} /^}/ {print name, ":", count, "instructions"}' "$last"
done

echo ""
echo "=== Total block parameters ==="
for branch_label in base current; do
    if [[ "$branch_label" == "base" ]]; then
        dir="$base_dir"
    else
        dir="$current_dir"
    fi

    last=$(ls "$dir"/*.ssa | tail -1)
    total=$(grep -oP 'b\d+\([^)]+\)' "$last" | awk -F',' '{print NF}' | paste -sd+ | bc)
    echo "$branch_label: $total block parameters"
done
