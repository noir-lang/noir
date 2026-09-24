#!/usr/bin/env bash
# Checks everything the Lean side of the integer-gadget pin promises:
#   1. every proof builds;
#   2. no proof escape hatch appears anywhere (`sorry`, custom axioms,
#      `native_decide`, kernel-check bypasses, ...);
#   3. the reviewed spec (`AcirLean/Spec/`) and the pinned data
#      (`AcirLean/Templates/`) never import unreviewed proof code, and the pinned
#      data contains plain definitions only;
#   4. `templates.golden` is exactly what the pinned templates print;
#   5. `Check.lean`: the final theorem proves exactly `AcirLean.AllClaims` from
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
if grep -rnE "$forbidden" AcirLean AcirLean.lean Check.lean EmitTemplates.lean; then
  fail "Forbidden proof escape hatch found."
fi

imports_outside() {
  local dir=$1 allowed=$2
  grep -hE '^import ' "$dir"/*.lean | awk '{print $2}' | grep -vE "$allowed" || true
}
bad=$(imports_outside AcirLean/Spec '^(Mathlib(\..*)?|AcirLean\.Spec\..*|AcirLean\.Templates\..*)$')
[ -z "$bad" ] || fail "AcirLean/Spec imports unreviewed modules: $bad"
bad=$(imports_outside AcirLean/Templates '^(Mathlib(\..*)?|AcirLean\.Spec\.(Semantics|Ssa)|AcirLean\.Templates\..*)$')
[ -z "$bad" ] || fail "AcirLean/Templates imports modules other than Mathlib, Spec.Semantics, Spec.Ssa and Templates: $bad"

templates_only_defs='^\s*(@\[|instance|notation|infix|infixl|infixr|prefix|postfix|macro|macro_rules|syntax|elab|attribute|set_option|open|local|scoped|theorem|lemma|opaque|partial|initialize|builtin_initialize)\b'
if grep -nE "$templates_only_defs" AcirLean/Templates/*.lean; then
  fail "AcirLean/Templates may only contain plain definitions."
fi

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

lake env lean Check.lean

echo "AllClaims proved from standard axioms; golden file current."
