#!/usr/bin/env bash
# Lean side of the integer-gadget pin. Fails unless every proof builds and
# templates.golden is exactly what the proved templates print.
set -euo pipefail
cd "$(dirname "$0")/.."
lake exe cache get
lake build
if grep -rnE '\b(sorry|admit|axiom)\b|native_decide' AcirLean; then
  echo "Forbidden proof-hole marker found." >&2
  exit 1
fi
generated=$(mktemp)
trap 'rm -f "$generated"' EXIT
lake env lean --run EmitTemplates.lean "$generated"
if ! cmp -s "$generated" templates.golden; then
  echo "templates.golden is not what the Lean-proved templates emit." >&2
  echo "Edit AcirLean/Template.lean (and its proofs), then regenerate with:" >&2
  echo "  lake env lean --run EmitTemplates.lean templates.golden" >&2
  diff "$generated" templates.golden | head -20 >&2
  exit 1
fi
echo "Lean templates proved and golden file current."
