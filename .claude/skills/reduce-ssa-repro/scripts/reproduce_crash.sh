#!/bin/bash
# Reproduce an SSA crash by validating the input, checking that the detection
# pass alone does not crash, then applying the full crashing pipeline.
#
# Usage:
#   SSA_PASSES="Unrolling" DETECTION_PASS="Inlining Brillig Calls" ./reproduce_crash.sh input.ssa
#
# Required environment variables:
#   SSA_PASSES        Colon-separated list of passes that trigger the bug
#   DETECTION_PASS    The pass that detects corruption (appended to pipeline)
#
# Optional environment variables:
#   NOIR_SSA          Path to noir-ssa binary (default: ../target/release/noir-ssa)
#
# The detection pass is always appended to the pipeline. It should be a pass
# that calls normalize_ids or otherwise validates SSA invariants.
# "Inlining Brillig Calls" is a good choice because it calls normalize_ids internally.
#
# Examples:
#   SSA_PASSES="Unrolling" DETECTION_PASS="Inlining Brillig Calls" ./reproduce_crash.sh input.ssa
#   SSA_PASSES="Constant Folding using constraints:Unrolling" DETECTION_PASS="Inlining Brillig Calls" ./reproduce_crash.sh input.ssa

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
NOIR_SSA="${NOIR_SSA:-$REPO_ROOT/target/release/noir-ssa}"
INPUT="${1:-./input.ssa}"

if [ -z "${DETECTION_PASS:-}" ]; then
    echo "Error: DETECTION_PASS must be set" >&2
    exit 1
fi

if [ -z "${SSA_PASSES:-}" ]; then
    echo "Error: SSA_PASSES must be set" >&2
    exit 1
fi

# Parse SSA_PASSES: colon-separated list of pass names
IFS=':' read -ra PASSES <<< "$SSA_PASSES"

echo "Using noir-ssa: $NOIR_SSA"
echo "Input SSA: $INPUT"
echo "Detection pass: $DETECTION_PASS"
echo "Pipeline: $(IFS=' → '; echo "${PASSES[*]}") → $DETECTION_PASS"
echo

# 1. Validate input parses
echo "Validating input SSA..."
"$NOIR_SSA" check --source-path "$INPUT" > /dev/null
echo "Input SSA is valid."
echo

# 2. Detection pass alone should not crash (proves bug is introduced by preceding passes)
echo "Checking that $DETECTION_PASS alone does not crash..."
"$NOIR_SSA" transform --source-path "$INPUT" \
  --ssa-pass "$DETECTION_PASS" \
  -o /dev/null
echo "OK — crash is introduced by the preceding passes, not pre-existing."
echo

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
