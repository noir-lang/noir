#!/usr/bin/env bash
set -euo pipefail

# Usage: average-cargo-bench.sh <bencher-output>...
#
# Prints one `test <name> ... bench: <value> <unit> (+/- <range>)` line per benchmark that appears
# in every given `cargo bench --output-format bencher` output: the mean of its values, and the
# largest of its ranges. Benchmarks missing from any of the outputs are left out.

# POSIX awk only: the benchmark machine's `awk` is mawk, which has no capture groups.
awk -v files="$#" '
  /^test [^ ]+ +\.\.\. bench: +[0-9,.]+ [a-z]+\/[a-z]+ \(\+\/- [0-9,.]+\)$/ {
    name = $2
    value = $5; gsub(",", "", value)
    range = $8; gsub("[,)]", "", range)
    if (!(name in count)) order[++n] = name
    count[name]++
    sum[name] += value
    unit[name] = $6
    if (range + 0 > max[name] + 0) max[name] = range
  }
  END {
    for (i = 1; i <= n; i++) {
      name = order[i]
      if (count[name] == files)
        printf "test %s ... bench: %.0f %s (+/- %.0f)\n", name, sum[name] / files, unit[name], max[name]
    }
  }' "$@"
