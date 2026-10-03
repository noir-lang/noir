#!/usr/bin/env bash
# Checks everything the Lean side of the integer-gadget pin promises:
#   1. every proof builds;
#   2. no proof escape hatch appears anywhere (`sorry`, custom axioms,
#      `native_decide`, kernel-check bypasses, ...);
#   3. the reviewed spec (`AcirLean/Spec/`) and the pinned data
#      (`AcirLean/Templates/`) never import unreviewed proof code, the pinned
#      data contains plain definitions only (`scripts/check_templates.py`), and
#      `lakefile.toml` passes no options to Lean;
#   4. `templates.golden` is exactly what the pinned templates print,
#      `test_programs.golden` exactly what the pinned test programs print, and
#      `ssa_semantics.golden` exactly what the SSA meaning computes on its grid;
#   5. `REVIEWING.md` quotes the reviewed Lean verbatim and mentions each of
#      its definitions (`scripts/check_reviewing.py`);
#   6. `Check.lean`: the final theorem proves exactly `AcirLean.AllClaims` from
#      Lean's three standard axioms.
set -euo pipefail
cd "$(dirname "$0")/.."

fail() {
  echo "$1" >&2
  exit 1
}

lake exe cache get
lake build

forbidden='\b(sorry|admit|axiom)\b|native_decide|skipKernelTC|implemented_by|@\[extern|\bunsafe\b|\bdebug\.'
if grep -rnE "$forbidden" AcirLean AcirLean.lean Check.lean EmitTemplates.lean EmitPrograms.lean EmitSemantics.lean lakefile.toml; then
  fail "Forbidden proof escape hatch found."
fi
# Options set here would apply to every module `lake build` compiles, and
# `#print axioms` in Check.lean would not show them.
if grep -nE '(leanOptions|moreLeanArgs|weakLeanArgs|moreServerOptions|moreGlobalServerArgs)' lakefile.toml; then
  fail "lakefile.toml may not pass options or arguments to Lean."
fi

imports_outside() {
  local dir=$1 allowed=$2
  grep -hE '^import ' "$dir"/*.lean | awk '{print $2}' | grep -vE "$allowed" || true
}
bad=$(imports_outside AcirLean/Spec '^(Mathlib(\..*)?|AcirLean\.Spec\..*|AcirLean\.Templates\..*)$')
[ -z "$bad" ] || fail "AcirLean/Spec imports unreviewed modules: $bad"
bad=$(imports_outside AcirLean/Templates '^(Mathlib(\..*)?|AcirLean\.Spec\.(Semantics|Ssa|Corpus|SsaSemantics)|AcirLean\.Templates\..*)$')
[ -z "$bad" ] || fail "AcirLean/Templates imports modules other than Mathlib, Spec.Semantics, Spec.Ssa, Spec.Corpus, Spec.SsaSemantics and Templates: $bad"

python3 scripts/check_templates.py

generated=$(mktemp)
trap 'rm -f "$generated"' EXIT
lake env lean --run EmitTemplates.lean "$generated"
if ! cmp -s "$generated" templates.golden; then
  echo "templates.golden is not what the Lean templates emit." >&2
  echo "Edit AcirLean/Templates/Gadgets.lean (and the proofs), then regenerate with:" >&2
  echo "  lake env lean --run EmitTemplates.lean templates.golden" >&2
  diff "$generated" templates.golden | head -20 >&2
  exit 1
fi

lake env lean --run EmitPrograms.lean "$generated"
if ! cmp -s "$generated" test_programs.golden; then
  echo "test_programs.golden is not what AcirLean/Templates/TestPrograms.lean emits." >&2
  echo "Regenerate both with scripts/regen_programs.sh." >&2
  diff "$generated" test_programs.golden | head -20 >&2
  exit 1
fi

lake env lean --run EmitSemantics.lean "$generated"
if ! cmp -s "$generated" ssa_semantics.golden; then
  echo "ssa_semantics.golden is not what EmitSemantics.lean emits." >&2
  echo "Regenerate it with:" >&2
  echo "  lake env lean --run EmitSemantics.lean ssa_semantics.golden" >&2
  echo "then run the Rust test fv_semantics to compare it with Noir's SSA interpreter." >&2
  diff "$generated" ssa_semantics.golden | head -20 >&2
  exit 1
fi

python3 scripts/check_reviewing.py

lake env lean Check.lean

echo "AllClaims proved from standard axioms; golden files current."
