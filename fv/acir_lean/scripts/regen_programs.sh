#!/usr/bin/env bash
# Regenerates the scalar test-program data from `test_programs/execution_success`:
# compiles every program with `nargo`, keeps each one's final SSA and shipped
# circuit, and writes
#   AcirLean/Templates/TestPrograms.lean  the programs in the scalar subset,
#   test_programs.golden                   what Lean must print for them,
#   test_programs.outside                  the others, with the reason.
# `scripts/check.sh` then requires the Lean data to print exactly
# `test_programs.golden`. With FV_PINNED=1 it rebuilds only the programs
# `testProgramNames` (`AcirLean/Spec/Coverage.lean`) lists, and fails
# if any of them cannot be rebuilt (see `gen_programs.py`). FV_WORK=<dir>
# keeps the compiled dumps there. Needs `cargo` and `python3`.
set -euo pipefail
cd "$(dirname "$0")/.."
root=$(cd ../.. && pwd)
target=${CARGO_TARGET_DIR:-$root/target}
if [ -n "${FV_WORK:-}" ]; then
  work=$FV_WORK
  mkdir -p "$work"
else
  work=$(mktemp -d)
  trap 'rm -rf "$work"' EXIT
fi

(cd "$root" && cargo build --release -p nargo_cli)
nargo=$target/release/nargo

: > "$work/artifacts.txt"
for dir in "$root"/test_programs/execution_success/*/; do
  name=$(basename "$dir")
  rm -f "$dir/target/$name.gz"
  if (cd "$dir" && "$nargo" compile --force \
      --show-ssa-pass "Mutable Array Set Optimizations" > "$work/$name.ssa" 2> /dev/null); then
    [ -f "$dir/target/$name.json" ] && echo "$dir/target/$name.json" >> "$work/artifacts.txt"
    # The witness ACVM solves from Prover.toml, for the claim that the circuit is satisfiable.
    (cd "$dir" && "$nargo" execute > /dev/null 2>&1) || true
  fi
done

(cd "$root" && FV_ARTIFACTS="$work/artifacts.txt" cargo test -q -p noirc_evaluator --lib \
  acir::acir_context::fv_templates::dump_artifacts -- --ignored --nocapture --exact) |
  grep -E '^(# artifact |zero |range |other |inputs |returns |solved$|witness )' > "$work/circuits.txt"

python3 scripts/gen_programs.py "$work" "$work/circuits.txt" test_programs.outside \
  AcirLean/Templates/TestPrograms.lean test_programs.golden

# The checker's certificates, when Lean is installed (`check.sh` requires them
# to be current).
if command -v lake > /dev/null; then
  lake build AcirLean.Proofs.Checker2 AcirLean.Templates.TestPrograms
  lake env lean --run scripts/emit_certs.lean AcirLean/Proofs/TestProgramCerts.lean
fi
