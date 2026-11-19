#!/usr/bin/env bash
set -euo pipefail

# Usage: critical-library-list.sh timed|untimed|all
#
# Prints the critical libraries of `EXTERNAL_NOIR_LIBRARIES.yml` as a JSON array, for a workflow's
# test matrix. A library is timed when its `timeout` is above 20 seconds: `reports.yml` checks and
# times those, and `test-js-packages.yml` checks the rest. The merge queue checks them all in
# `test-js-packages.yml`, since `reports.yml` doesn't run there.

KIND=$1
case $KIND in
  timed | untimed | all) ;;
  *)
    echo "usage: $0 timed|untimed|all" >&2
    exit 64
    ;;
esac

LIBRARIES=$(yq ./EXTERNAL_NOIR_LIBRARIES.yml -o json | jq -c --arg kind "$KIND" '
  .libraries
  | map({ repo, ref, path: (.path // ""), timeout: (.timeout // 5), nargo_args })
  | map(select($kind == "all" or ((.timeout > 20) == ($kind == "timed"))))')
if [ -z "$LIBRARIES" ] || [ "$LIBRARIES" = "null" ]; then
  echo "::error::Failed to parse EXTERNAL_NOIR_LIBRARIES.yml — got empty or null library list" >&2
  exit 1
fi
if [ "$(jq 'length' <<< "$LIBRARIES")" -eq 0 ]; then
  echo "::error::EXTERNAL_NOIR_LIBRARIES.yml has no $KIND libraries" >&2
  exit 1
fi
echo "$LIBRARIES"
