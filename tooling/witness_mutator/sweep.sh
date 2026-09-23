#!/usr/bin/env bash
# Run the witness mutator over every program in a test_programs directory.
#
# Usage: tooling/witness_mutator/sweep.sh [test-programs-subdir] [jobs] [per-program-timeout]
#
# Writes one TSV line per program to stdout: <name> <verdict> <detail>, where verdict is one of
# HIGH (a compiler-inserted hint is underconstrained), PROGRAM (the program returns an
# unconstrained call's output without checking it), WITNESS (no return value moves but the rest of
# the witness does), INERT (the free value is read by nothing), none, timeout, or
# skipped (no Prover.toml, or honest execution failed — programs needing an oracle cannot be
# searched without their foreign call transcript).
set -uo pipefail

subdir=${1:-execution_success}
jobs=${2:-16}
per_program_timeout=${3:-120}

root=$(git rev-parse --show-toplevel)
export root per_program_timeout
export nargo="$root/target/debug/nargo"
export mutator="$root/target/debug/noir-witness-mutator"
export max_candidates=${MAX_CANDIDATES:-2000}

one() {
  local dir=$1 name
  name=$(basename "$dir")
  [ -f "$dir/Prover.toml" ] || { printf '%s\tskipped\tno Prover.toml\n' "$name"; return; }

  rm -rf "$dir/target"
  if ! timeout "$per_program_timeout" "$nargo" compile --program-dir "$dir" >/dev/null 2>&1; then
    printf '%s\tskipped\tcompile failed\n' "$name"
    return
  fi
  local artifact
  artifact=$(find "$dir/target" -name '*.json' | head -1)
  [ -n "$artifact" ] || { printf '%s\tskipped\tno artifact\n' "$name"; return; }

  local output status
  output=$(timeout "$per_program_timeout" "$mutator" --artifact-path "$artifact" \
    --prover-file "$dir/Prover.toml" --max-candidates "$max_candidates" 2>&1)
  status=$?
  rm -rf "$dir/target"

  if [ $status -eq 124 ]; then
    printf '%s\ttimeout\t%ss\n' "$name" "$per_program_timeout"
  elif grep -q '^HIGH' <<<"$output"; then
    printf '%s\tHIGH\t%s\n' "$name" "$(grep '^HIGH' <<<"$output" | head -1)"
  elif grep -q '^PROGRAM' <<<"$output"; then
    printf '%s\tPROGRAM\t%s\n' "$name" "$(grep '^PROGRAM' <<<"$output" | head -1)"
  elif grep -q '^WITNESS' <<<"$output"; then
    printf '%s\tWITNESS\t%s\n' "$name" "$(grep '^WITNESS' <<<"$output" | head -1)"
  elif grep -q '^INERT' <<<"$output"; then
    printf '%s\tINERT\t%s\n' "$name" "$(grep '^INERT' <<<"$output" | head -1)"
  elif grep -q 'honest execution failed' <<<"$output"; then
    printf '%s\tskipped\thonest execution failed\n' "$name"
  else
    printf '%s\tnone\t%s\n' "$name" "$(head -1 <<<"$output")"
  fi
}
export -f one

find "$root/test_programs/$subdir" -mindepth 1 -maxdepth 1 -type d -print0 |
  sort -z |
  xargs -0 -P "$jobs" -I{} bash -c 'one "$@"' _ {}
