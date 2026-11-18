#!/usr/bin/env bash
set -eu

# Usage: cargo-bench-to-json.sh <bencher-output>
#
# Prints the `test <name> ... bench: <value> <unit> (+/- <range>)` lines of `cargo bench` output in
# `--output-format bencher` as a JSON array of `{ name, value, range, unit }` benches, the shape
# github-action-benchmark's `cargo` tool stores them in.

sed -nE 's/^test (.+[^ ]) +\.\.\. bench: +([0-9,.]+) ([a-z]+\/[a-z]+) \(\+\/- ([0-9,.]+)\)$/\1\t\2\t\3\t\4/p' "$1" |
  jq --raw-input --slurp '
    split("\n") | map(select(length > 0) | split("\t") | {
      name: .[0],
      value: (.[1] | gsub(","; "") | tonumber),
      range: ("± " + (.[3] | gsub(","; ""))),
      unit: .[2]
    })'
