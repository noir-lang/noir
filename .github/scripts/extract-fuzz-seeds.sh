#!/bin/bash
set -eu

# Usage: ./extract-fuzz-seeds.sh <fuzz-output.log> [output-dir]
#
# Parses a captured `cargo nextest run -p noir_ast_fuzzer_fuzz` log and writes:
#   <output-dir>/seeds.txt          — one deduplicated 0x... seed per line
#   <output-dir>/seeds-by-test.tsv  — "<nextest FAIL line>\t<seed>" pairs
#
# Recognises both the arbtest panic line (e.g. "    Seed: 0x6819c61400001000")
# and the explicit repro env var (e.g. "NOIR_AST_FUZZER_SEED=0x6819c61400001000").

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Usage: $0 <fuzz-output.log> [output-dir]" >&2
    exit 2
fi

log_file=$1
out_dir=${2:-.}

if [ ! -f "$log_file" ]; then
    echo "error: log file not found: $log_file" >&2
    exit 1
fi

mkdir -p "$out_dir"
seeds_file="$out_dir/seeds.txt"
by_test_file="$out_dir/seeds-by-test.tsv"

grep -hoE '(Seed:[[:space:]]*0x[0-9a-fA-F]+|NOIR_AST_FUZZER_SEED=0x[0-9a-fA-F]+)' "$log_file" \
    | grep -oE '0x[0-9a-fA-F]+' \
    | sort -u > "$seeds_file" || true

awk '
    /^[[:space:]]*FAIL / { current = $0 }
    /Seed:[[:space:]]*0x[0-9a-fA-F]+/ {
        match($0, /0x[0-9a-fA-F]+/); seed = substr($0, RSTART, RLENGTH);
        print current "\t" seed
    }
' "$log_file" > "$by_test_file" || true
