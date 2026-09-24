#!/usr/bin/env bash
# Regenerates the scalar test-program data from `test_programs/execution_success`:
# compiles every program with `nargo`, keeps each one's final SSA and shipped
# circuit, and writes
#   AcirLean/Templates/TestPrograms.lean  the programs in the scalar subset,
#   test_programs.golden                   what Lean must print for them,
#   test_programs.outside                  the others, with the reason.
# `scripts/check.sh` then requires the Lean data to print exactly
# `test_programs.golden`. Needs `cargo` and `python3`.
set -euo pipefail
cd "$(dirname "$0")/.."
root=$(cd ../.. && pwd)
target=${CARGO_TARGET_DIR:-$root/target}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

(cd "$root" && cargo build --release -p nargo_cli)
nargo=$target/release/nargo

: > "$work/artifacts.txt"
for dir in "$root"/test_programs/execution_success/*/; do
  name=$(basename "$dir")
  if (cd "$dir" && "$nargo" compile --force \
      --show-ssa-pass "Mutable Array Set Optimizations" > "$work/$name.ssa" 2> /dev/null); then
    [ -f "$dir/target/$name.json" ] && echo "$dir/target/$name.json" >> "$work/artifacts.txt"
  fi
done

(cd "$root" && FV_ARTIFACTS="$work/artifacts.txt" cargo test -q -p noirc_evaluator --lib \
  acir::acir_context::fv_templates::dump_artifacts -- --ignored --nocapture --exact) |
  grep -E '^(# artifact |zero |range |other |inputs |returns )' > "$work/circuits.txt"

python3 scripts/gen_programs.py "$work" "$work/circuits.txt" test_programs.outside \
  AcirLean/Templates/TestPrograms.lean test_programs.golden
