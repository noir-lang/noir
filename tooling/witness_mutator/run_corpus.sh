#!/usr/bin/env bash
# Run the witness mutator over one corpus entry, with and without its bug patch.
#
# Usage: tooling/witness_mutator/run_corpus.sh <entry-name>
#
# The compiler is rebuilt for each side, and every program's `target/` is cleared before it is
# compiled: nargo caches the compiled artifact per program directory, so a leftover artifact would
# silently be measured against the wrong source tree.
set -euo pipefail

entry=${1:?usage: run_corpus.sh <entry-name>}
root=$(git rev-parse --show-toplevel)
dir="$root/tooling/witness_mutator/corpus/$entry"
program="$dir/program"
[ -d "$program" ] || { echo "$entry has no trigger program"; exit 2; }

cd "$root"
mutator="$root/target/debug/noir-witness-mutator"
nargo="$root/target/debug/nargo"

sweep() {
  rm -rf "$program/target"
  "$nargo" compile --program-dir "$program" >/dev/null
  local artifact
  artifact=$(find "$program/target" -name '*.json' | head -1)
  local inputs=("$program"/inputs/*.toml)
  [ -e "${inputs[0]}" ] || inputs=("$program/Prover.toml")

  local high=0 total=0 output
  for input in "${inputs[@]}"; do
    total=$((total + 1))
    # The mutator exits non-zero on a HIGH finding, which is the interesting case rather than an
    # error, so its status is captured instead of ending the run.
    output=$("$mutator" --artifact-path "$artifact" --prover-file "$input" || true)
    case "$output" in
      HIGH*|*$'\n'HIGH*) high=$((high + 1)) ;;
    esac
  done
  echo "$high/$total"
}

cargo build -p nargo_cli --bin nargo -p noir_witness_mutator >/dev/null 2>&1
echo "clean master: $(sweep) inputs with a second witness"

git apply "$dir/bug.patch"
trap 'git checkout -- compiler acvm-repo' EXIT
cargo build -p nargo_cli --bin nargo >/dev/null 2>&1
echo "bug applied:  $(sweep) inputs with a second witness"
