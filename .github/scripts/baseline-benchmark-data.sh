#!/usr/bin/env bash
set -eu

# Usage: baseline-benchmark-data.sh <name> <sha> <metric> <report>...
#
# Prints benchmark data in github-action-benchmark's format, holding a single result for the
# benchmark `<name>`: the base branch's, at commit `<sha>`. Each report is a JSON object whose
# `<metric>` field is a `{ name, value, unit }` bench, as written by `process_benchmark_data.sh`.
# With an empty `<metric>`, each report is instead a JSON array of such benches.
#
# Passed as `external-data-json-path`, it makes the action compare a PR against its base branch
# measured in the same jobs, so on the same machines, rather than against master's history.

NAME=$1
SHA=$2
METRIC=$3
shift 3

# Read through `cat` with stdin closed, so that no reports at all gives no benches instead of `jq`
# waiting on stdin.
cat -- "$@" < /dev/null | jq --slurp --arg name "$NAME" --arg sha "$SHA" --arg metric "$METRIC" '
  (if $metric == "" then flatten else map(.[$metric] | select(.)) end) as $benches
  | {
    lastUpdate: 0,
    repoUrl: "",
    entries: {
      ($name): [
        {
          commit: {
            id: $sha,
            message: "",
            timestamp: "",
            url: "",
            author: { name: "", username: "" },
            committer: { name: "", username: "" }
          },
          date: 0,
          tool: "customSmallerIsBetter",
          benches: $benches
        }
      ]
    }
  }'
