#!/usr/bin/env bash
# Split SSA output into separate files, one per pass.
# Usage: split-ssa-passes.sh <ssa_output_file> [output_dir]
#
# The output directory defaults to "ssa_passes".
# Files are named with numeric prefixes to preserve pass order.

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <ssa_output_file> [output_dir]" >&2
    exit 1
fi

input_file="$1"
output_dir="${2:-ssa_passes}"

mkdir -p "$output_dir"

awk -v outdir="$output_dir" '
BEGIN { file_num = 0; current_file = "" }
/^After / {
    if (current_file != "") close(current_file)
    file_num++
    pass_name = $0
    gsub(/^After /, "", pass_name)
    gsub(/:$/, "", pass_name)
    gsub(/[^a-zA-Z0-9_() -]/, "", pass_name)
    gsub(/ +/, "_", pass_name)
    current_file = sprintf("%s/%02d_%s.ssa", outdir, file_num, pass_name)
    print "Created: " current_file > "/dev/stderr"
}
{ if (current_file != "") print > current_file }
' "$input_file"

echo "Split into $output_dir/"
