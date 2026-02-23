#!/bin/bash
# Reproduce an SSA crash by validating the input, then applying the crashing pipeline.
#
# Two modes:
#   Multi-pass mode: SSA_PASSES set — validates that (1) input parses, (2) detection
#     pass alone does not crash, and (3) the full pipeline crashes.
#   Single-pass mode: SSA_PASSES unset/empty — validates that (1) input parses and
#     (2) the crashing pass crashes. Use when a single pass crashes on the input
#     directly, with no preceding passes needed to introduce corruption.
#
# Usage:
#   # Multi-pass: corruption passes + detection pass
#   SSA_PASSES="Unrolling" DETECTION_PASS="Inlining Brillig Calls" ./reproduce_crash.sh input.ssa
#   SSA_PASSES="Constant Folding using constraints:Unrolling" DETECTION_PASS="Inlining Brillig Calls" ./reproduce_crash.sh input.ssa
#
#   # Single-pass: the pass itself crashes on the input
#   DETECTION_PASS="Unrolling" ./reproduce_crash.sh input.ssa
#
# Required environment variables:
#   DETECTION_PASS    The pass that detects corruption (multi-pass) or crashes (single-pass)
#
# Optional environment variables:
#   SSA_PASSES        Colon-separated list of passes that trigger the bug (omit for single-pass mode)
#   NOIR_SSA          Path to noir-ssa binary (default: ../target/debug/noir-ssa)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
NOIR_SSA="${NOIR_SSA:-$REPO_ROOT/target/debug/noir-ssa}"
INPUT="${1:-./input.ssa}"

if [ -z "${DETECTION_PASS:-}" ]; then
    echo "Error: DETECTION_PASS must be set" >&2
    exit 1
fi

# Parse SSA_PASSES: colon-separated list of pass names (may be empty)
PASSES=()
if [ -n "${SSA_PASSES:-}" ]; then
    IFS=':' read -ra PASSES <<< "$SSA_PASSES"
fi

echo "Using noir-ssa: $NOIR_SSA"
echo "Input SSA: $INPUT"
if [ ${#PASSES[@]} -gt 0 ]; then
    echo "Pipeline: $(IFS=' → '; echo "${PASSES[*]}") → $DETECTION_PASS"
else
    echo "Crashing pass: $DETECTION_PASS (single-pass mode)"
fi
echo

# 1. Validate input parses
echo "Validating input SSA..."
"$NOIR_SSA" check --source-path "$INPUT" > /dev/null
echo "Input SSA is valid."
echo

# 2. In multi-pass mode, detection pass alone should not crash
#    (proves bug is introduced by preceding passes, not pre-existing)
if [ ${#PASSES[@]} -gt 0 ]; then
    echo "Checking that $DETECTION_PASS alone does not crash..."
    "$NOIR_SSA" transform --source-path "$INPUT" \
      --ssa-pass "$DETECTION_PASS" \
      -o /dev/null
    echo "OK — crash is introduced by the preceding passes, not pre-existing."
    echo
else
    echo "Single-pass mode — skipping detection-only check."
    echo
fi

# 3. Apply the crashing pipeline
echo "Applying crashing pipeline..."
PASS_ARGS=()
for pass in "${PASSES[@]}"; do
    PASS_ARGS+=(--ssa-pass "$pass")
done
PASS_ARGS+=(--ssa-pass "$DETECTION_PASS")

"$NOIR_SSA" transform --source-path "$INPUT" \
  "${PASS_ARGS[@]}" \
  -o /dev/null
