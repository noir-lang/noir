#!/usr/bin/env bash
# Thin wrapper around .github/scripts/run-external-repo-tests.sh for local use.
# Sets up the env vars the CI script expects, then delegates to it.
#
# Usage:
#   ./run-external-repo-test.sh <repo> [ref] [path] [extra_nargo_args...]
#
# Examples:
#   ./run-external-repo-test.sh noir-lang/noir_bigcurve
#   ./run-external-repo-test.sh noir-lang/noir-bignum "" "" "--test-threads 1"
#   ./run-external-repo-test.sh AztecProtocol/aztec-packages 2b1671a noir-projects/aztec-nr
#
# Environment variables:
#   NARGO       - path to nargo binary (default: nargo on PATH)
#   CLONE_DIR   - where to clone repos (default: /tmp/external-repos)
#   KEEP_CLONE  - set to 1 to skip re-cloning if the repo dir already exists

set -eu

REPO="${1:?Usage: $0 <repo> [ref] [path] [extra_nargo_args...]}"
REF="${2:-}"
PROJECT_PATH="${3:-}"
ARGC=$#
if [ $ARGC -ge 3 ]; then
    shift 3
    EXTRA_ARGS="$*"
else
    EXTRA_ARGS=""
fi

# Resolve NARGO to absolute path so it works after cd
export NARGO="${NARGO:-nargo}"
if [[ "$NARGO" != /* ]]; then
    if [ -f "$NARGO" ]; then
        NARGO="$(realpath "$NARGO")"
    elif command -v "$NARGO" > /dev/null 2>&1; then
        NARGO="$(realpath "$(command -v "$NARGO")")"
    fi
fi
export NARGO

CLONE_DIR="${CLONE_DIR:-/tmp/external-repos}"
RESULTS_DIR="/tmp/external-test-results"

# Derive a safe slug from the repo name
SLUG="${REPO//\//_}"
if [ -n "$PROJECT_PATH" ]; then
    PATH_SLUG="${PROJECT_PATH//\//_}"
    SLUG="${SLUG}_${PATH_SLUG}"
fi

mkdir -p "$RESULTS_DIR"

# Clone or reuse the repo (the CI script clones when CI is unset, but we
# want persistent clones under CLONE_DIR for repeated runs)
REPO_DIR="$CLONE_DIR/$REPO"
if [ "${KEEP_CLONE:-0}" = "1" ] && [ -d "$REPO_DIR" ]; then
    echo "==> Reusing existing clone at $REPO_DIR"
else
    echo "==> Cloning https://github.com/$REPO (shallow)..."
    rm -rf "$REPO_DIR"
    mkdir -p "$(dirname "$REPO_DIR")"

    if [ -n "$REF" ]; then
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

# Find the repo root (where .github/scripts lives)
NOIR_REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CI_SCRIPT="$NOIR_REPO_ROOT/.github/scripts/run-external-repo-tests.sh"

if [ ! -f "$CI_SCRIPT" ]; then
    echo "ERROR: Cannot find CI script at $CI_SCRIPT"
    exit 1
fi

# Set env vars the CI script expects
export CI=1  # Prevent CI script from cloning (we already did)
export REPO_DIR="$REPO_DIR"
export PROJECT_PATH="${PROJECT_PATH}"
export NARGO_ARGS="${EXTRA_ARGS}"
export OUTPUT_FILE="$RESULTS_DIR/${SLUG}.jsonl"
export BENCHMARK_FILE="$RESULTS_DIR/${SLUG}.timing.json"
export NAME="$SLUG"

echo "==> Delegating to $CI_SCRIPT"
echo "    nargo: $NARGO"
echo "    repo:  $REPO_DIR/$PROJECT_PATH"
echo ""

exec "$CI_SCRIPT"
