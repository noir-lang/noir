#!/usr/bin/env bash
# Compare test timing between two nargo binaries for an external repo.
#
# Usage:
#   ./compare-external-repo-timing.sh <repo> [ref] [path] [extra_nargo_args...]
#
# Environment variables (required):
#   NARGO_BASELINE - path to the baseline nargo binary (e.g. from master)
#   NARGO_TEST     - path to the test nargo binary (e.g. from the PR branch)
#
# This script runs the test suite twice (once per binary) and prints a comparison.

set -eu

REPO="${1:?Usage: $0 <repo> [ref] [path] [extra_nargo_args...]}"

NARGO_BASELINE="${NARGO_BASELINE:?Set NARGO_BASELINE to the baseline nargo binary}"
NARGO_TEST="${NARGO_TEST:?Set NARGO_TEST to the test nargo binary}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESULTS_DIR="/tmp/external-test-results"

SLUG="${REPO//\//_}"

echo "===== BASELINE RUN ====="
NARGO="$NARGO_BASELINE" KEEP_CLONE=1 "$SCRIPT_DIR/run-external-repo-test.sh" "$@" || true
BASELINE_TIME=$(jq '.[0].value' "$RESULTS_DIR/${SLUG}.timing.json")
cp "$RESULTS_DIR/${SLUG}.jsonl" "$RESULTS_DIR/${SLUG}.baseline.jsonl" 2>/dev/null || true
cp "$RESULTS_DIR/${SLUG}.timing.json" "$RESULTS_DIR/${SLUG}.baseline.timing.json"

echo ""
echo "===== TEST RUN ====="
NARGO="$NARGO_TEST" KEEP_CLONE=1 "$SCRIPT_DIR/run-external-repo-test.sh" "$@" || true
TEST_TIME=$(jq '.[0].value' "$RESULTS_DIR/${SLUG}.timing.json")
cp "$RESULTS_DIR/${SLUG}.jsonl" "$RESULTS_DIR/${SLUG}.test.jsonl" 2>/dev/null || true
cp "$RESULTS_DIR/${SLUG}.timing.json" "$RESULTS_DIR/${SLUG}.test.timing.json"

echo ""
echo "============================================"
echo "  TIMING COMPARISON: $REPO"
echo "  Baseline: ${BASELINE_TIME}s"
echo "  Test:     ${TEST_TIME}s"

if [ "$BASELINE_TIME" -gt 0 ] 2>/dev/null; then
    RATIO=$(python3 -c "print(f'{($TEST_TIME / $BASELINE_TIME):.2f}')")
    CHANGE=$(python3 -c "print(f'{(($TEST_TIME - $BASELINE_TIME) * 100 / $BASELINE_TIME):.1f}')")
    echo "  Ratio:    ${RATIO}x"
    echo "  Change:   ${CHANGE}%"
fi
echo "============================================"
