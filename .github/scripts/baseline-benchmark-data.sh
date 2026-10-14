#!/usr/bin/env bash
set -eu

# Prints benchmark data in github-action-benchmark's format, holding a single result for the
# benchmark `$1`: the base branch's, at commit `$2`, read from `$3` (a JSON array of
# `{ name, value, unit }`).
#
# Passed as `external-data-json-path`, it makes the action compare a PR against its base branch
# measured in the same jobs, so on the same machines, rather than against master's history.

NAME=$1
SHA=$2
BENCHES=$3

jq --null-input --arg name "$NAME" --arg sha "$SHA" --slurpfile benches "$BENCHES" '
  {
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
          benches: $benches[0]
        }
      ]
    }
  }'
