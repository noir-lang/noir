#!/usr/bin/env bash
# Run tests for an external Noir library locally, mirroring CI behavior.
#
# Usage:
#   ./run-external-repo-test.sh <repo> [ref] [path] [extra_nargo_args...]
#
# Examples:
#   ./run-external-repo-test.sh noir-lang/noir_bigcurve
#   ./run-external-repo-test.sh noir-lang/noir-bignum main "" "--test-threads 1"
#   ./run-external-repo-test.sh AztecProtocol/aztec-packages 2b1671a noir-projects/aztec-nr
#
# Environment variables:
#   NARGO       - path to nargo binary (default: nargo on PATH)
#   CLONE_DIR   - where to clone repos (default: /tmp/external-repos)
#   KEEP_CLONE  - set to 1 to skip re-cloning if the repo dir already exists
#
# Output:
#   - Test results JSON:  /tmp/external-test-results/<repo_slug>.jsonl
#   - Timing JSON:        /tmp/external-test-results/<repo_slug>.timing.json
#   - Console output with clear timing summary

set -eu

REPO="${1:?Usage: $0 <repo> [ref] [path] [extra_nargo_args...]}"
REF="${2:-}"
PROJECT_PATH="${3:-}"
# Shift past the positional args we already captured
ARGC=$#
if [ $ARGC -ge 3 ]; then
    shift 3
    EXTRA_ARGS="$*"
else
    EXTRA_ARGS=""
fi

NARGO="${NARGO:-nargo}"
# Resolve to absolute path so it works after cd
if [[ "$NARGO" != /* ]]; then
    if [ -f "$NARGO" ]; then
        NARGO="$(realpath "$NARGO")"
    elif command -v "$NARGO" > /dev/null 2>&1; then
        NARGO="$(realpath "$(command -v "$NARGO")")"
    fi
fi
CLONE_DIR="${CLONE_DIR:-/tmp/external-repos}"
RESULTS_DIR="/tmp/external-test-results"

# Derive a safe slug from the repo name
SLUG="${REPO//\//_}"
if [ -n "$PROJECT_PATH" ]; then
    PATH_SLUG="${PROJECT_PATH//\//_}"
    SLUG="${SLUG}_${PATH_SLUG}"
fi

mkdir -p "$RESULTS_DIR"
OUTPUT_FILE="$RESULTS_DIR/${SLUG}.jsonl"
TIMING_FILE="$RESULTS_DIR/${SLUG}.timing.json"

# Clone or reuse the repo
REPO_DIR="$CLONE_DIR/$REPO"
if [ "${KEEP_CLONE:-0}" = "1" ] && [ -d "$REPO_DIR" ]; then
    echo "==> Reusing existing clone at $REPO_DIR"
else
    echo "==> Cloning https://github.com/$REPO (shallow)..."
    rm -rf "$REPO_DIR"
    mkdir -p "$(dirname "$REPO_DIR")"

    if [ -n "$REF" ]; then
        # Try shallow clone with the ref first; fall back to full clone for commit hashes
        if git clone --depth 1 --branch "$REF" "https://github.com/$REPO.git" "$REPO_DIR" 2>/dev/null; then
            true
        else
            echo "    Shallow clone failed (probably a commit hash), doing full clone..."
            git clone "https://github.com/$REPO.git" "$REPO_DIR"
            git -C "$REPO_DIR" -c advice.detachedHead=false checkout "$REF"
        fi
    else
        git clone --depth 1 "https://github.com/$REPO.git" "$REPO_DIR"
    fi
fi

# Enter the project directory
WORK_DIR="$REPO_DIR"
if [ -n "$PROJECT_PATH" ]; then
    WORK_DIR="$REPO_DIR/$PROJECT_PATH"
fi
cd "$WORK_DIR"

# Strip compiler_version from Nargo.toml files (just like CI does)
set +e
sed -i '/^compiler_version/d' Nargo.toml ./**/Nargo.toml 2>/dev/null
set -e

echo "==> Running tests in $WORK_DIR"
echo "    nargo: $NARGO"
echo "    args:  --silence-warnings --skip-brillig-constraints-check --format json $EXTRA_ARGS"
echo ""

# Run tests with timing
BEFORE=$SECONDS
set +e
$NARGO test --silence-warnings --skip-brillig-constraints-check --format json $EXTRA_ARGS 2>&1 | tee "$OUTPUT_FILE"
TEST_EXIT=${PIPESTATUS[0]}
set -e
ELAPSED=$(($SECONDS - $BEFORE))

# Write timing file
jq --null-input "[{ name: \"$SLUG\", value: (\"$ELAPSED\" | tonumber), unit: \"s\" }]" > "$TIMING_FILE"

echo ""
echo "============================================"
echo "  Test suite: $REPO${PROJECT_PATH:+/$PROJECT_PATH}"
echo "  Exit code:  $TEST_EXIT"
echo "  Duration:   ${ELAPSED}s"
echo "  Results:    $OUTPUT_FILE"
echo "  Timing:     $TIMING_FILE"
echo "============================================"

exit $TEST_EXIT
