#!/usr/bin/env bash
set -uo pipefail

# Usage: ./check-nargo-stderr.sh <expected-stderr> <actual-stderr> <command> [args...]
#
# Runs <command> and compares its normalized stderr against the snapshot in <expected-stderr>.
# The exit status is not required to be zero: a PR that is expected to break an external project
# records the resulting errors in the snapshot instead of disabling the check. The normalized
# stderr is written to <actual-stderr> so that it can be copied into the snapshot.
#
# Normalization makes the output independent of the machine and of how many workspace members
# share a dependency:
# - `STRIP_PREFIX`, when set, is removed from every line (absolute paths to the checkout);
# - ANSI colour codes are removed;
# - nargo's `Cloning into '...'...` progress lines for git dependencies are dropped;
# - the `Aborting due to N previous errors` summary is dropped, as N counts repeated diagnostics;
# - repeated diagnostics are kept once. Each workspace member elaborates its dependencies on its
#   own, so an error in a shared dependency is otherwise reported once per member.

expected=$1
actual=$2
shift 2

raw=$(mktemp)
"$@" 2> "$raw" > /dev/null
status=$?

mkdir -p "$(dirname "$actual")"
STRIP_PREFIX=${STRIP_PREFIX:-} perl -pe 's/\e\[[0-9;]*m//g; s/\Q$ENV{STRIP_PREFIX}\E//g if length $ENV{STRIP_PREFIX}' "$raw" \
  | grep -vE "^Cloning into '.*'\.\.\.$" \
  | grep -vE '^Aborting due to [0-9]+ previous errors?$' \
  | awk 'BEGIN { RS = ""; ORS = "\n\n" } !seen[$0]++' \
  > "$actual"

if [ "$status" -ne 0 ] && [ ! -s "$actual" ]; then
  echo "Error: \`$*\` exited with status $status without reporting any diagnostics. Its stderr was:"
  cat "$raw"
  exit 1
fi

if ! diff -u "$expected" "$actual"; then
  echo "Error: the diagnostics don't match the snapshot in '$expected'."
  echo "Lines prefixed with '+' are new diagnostics, lines prefixed with '-' are expected diagnostics that are no longer reported."
  echo "If this change is intended, replace the contents of '$expected' with the new output (also uploaded as this job's artifact)."
  exit 1
fi
