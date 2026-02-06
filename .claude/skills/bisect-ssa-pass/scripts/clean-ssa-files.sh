#!/usr/bin/env bash
# Clean SSA files by removing headers and diagnostics.
# Usage: clean-ssa-files.sh [ssa_dir]
#
# Removes:
#   - "After ..." header lines from all files
#   - Compiler diagnostics (warnings/errors) from the final file

set -euo pipefail

ssa_dir="${1:-ssa_passes}"

if [[ ! -d "$ssa_dir" ]]; then
    echo "Error: Directory '$ssa_dir' not found" >&2
    exit 1
fi

# Strip header lines from all files
for f in "$ssa_dir"/*.ssa; do
    [[ -f "$f" ]] || continue
    tail -n +2 "$f" | sed '/^$/N;/^\n$/d' > "$f.tmp" && mv "$f.tmp" "$f"
done
echo "Stripped headers from all .ssa files"

# Strip diagnostics from the final file
last_file=$(ls "$ssa_dir"/*.ssa 2>/dev/null | tail -1)
if [[ -n "$last_file" && -f "$last_file" ]]; then
    awk '/^warning:|^error:/ {exit} {print}' "$last_file" > "$last_file.tmp" && mv "$last_file.tmp" "$last_file"
    echo "Stripped diagnostics from $(basename "$last_file")"
fi
