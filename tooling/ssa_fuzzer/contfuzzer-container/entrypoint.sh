#!/usr/bin/env bash
set -euo pipefail

# Platform runs as UID 65534:GID 10001 — ensure files written to bind mounts
# (/crashes, /corpus, /output) are group-accessible for the worker process.
umask 000

# ContFuzzer SSA fuzzer entrypoint (multi-variant).
#
# Two build variants in /targets/:
#   /targets/fuzzing/<target>   — ASan + libFuzzer (fuzz, minimize, reproduce, regress)
#   /targets/coverage/<target>  — coverage-instrumented (coverage mode → profraw → llvm-cov)
#
# Platform contract (FUZZ_* env vars):
#   FUZZ_MODE          fuzz | coverage | minimize | reproduce | regress
#   FUZZ_TARGET        target name (e.g. acir_vs_brillig)
#   FUZZ_CORPUS_DIR    /corpus
#   FUZZ_CRASH_DIR     /crashes
#   FUZZ_OUTPUT_DIR    /output
#   FUZZ_TIMEOUT       seconds
#   FUZZ_JOBS          parallel workers
#   FUZZ_MEMORY        memory limit e.g. "8g"
#   FUZZ_REPRODUCE_DIR /reproduce-input (reproduce mode only)
#   FUZZ_CRASH_FILE    path to crash file (reproduce mode only)

: "${FUZZ_MODE:=fuzz}"
: "${FUZZ_TARGET:=}"
: "${FUZZ_CORPUS_DIR:=/corpus}"
: "${FUZZ_CRASH_DIR:=/crashes}"
: "${FUZZ_OUTPUT_DIR:=/output}"
: "${FUZZ_TIMEOUT:=0}"
: "${FUZZ_JOBS:=1}"
: "${FUZZ_MEMORY:=}"
: "${FUZZ_REPRODUCE_DIR:=}"
: "${FUZZ_CRASH_FILE:=}"

_parse_memory_mb() {
    local mem="${1:-}"
    case "$mem" in
        *g) echo $(( ${mem%g} * 1024 )) ;;
        *m) echo "${mem%m}" ;;
        *)  echo "" ;;
    esac
}

# ── Binary resolution ──────────────────────────────────────────────────
# Select build variant based on FUZZ_MODE.
case "$FUZZ_MODE" in
    fuzz|minimize|regress|reproduce) VARIANT="fuzzing" ;;
    coverage)                        VARIANT="coverage" ;;
    *)
        echo "ERROR: Unknown FUZZ_MODE=$FUZZ_MODE" >&2
        exit 2
        ;;
esac

if [ -z "$FUZZ_TARGET" ]; then
    echo "ERROR: FUZZ_TARGET is required" >&2
    exit 2
fi

BINARY="/targets/$VARIANT/$FUZZ_TARGET"
if [ ! -x "$BINARY" ]; then
    echo "ERROR: Binary not found: $BINARY" >&2
    echo "Available in /targets/$VARIANT/:" >&2
    ls -1 "/targets/$VARIANT/" 2>/dev/null || echo "  (none)" >&2
    exit 2
fi

mkdir -p "$FUZZ_CORPUS_DIR" "$FUZZ_CRASH_DIR" "$FUZZ_OUTPUT_DIR" 2>/dev/null || true

# ── Build command arguments ────────────────────────────────────────────
case "$FUZZ_MODE" in
    fuzz)
        ARGS=()
        ARGS+=("$FUZZ_CORPUS_DIR")
        ARGS+=("-artifact_prefix=$FUZZ_CRASH_DIR/")
        if [ "$FUZZ_TIMEOUT" -gt 0 ] 2>/dev/null; then
            ARGS+=("-max_total_time=$FUZZ_TIMEOUT")
        fi
        if [ "$FUZZ_JOBS" -gt 1 ] 2>/dev/null; then
            ARGS+=("-jobs=$FUZZ_JOBS" "-workers=$FUZZ_JOBS")
        fi
        if [ -n "$FUZZ_MEMORY" ]; then
            RSS_MB=$(_parse_memory_mb "$FUZZ_MEMORY")
            [ -n "$RSS_MB" ] && ARGS+=("-rss_limit_mb=$RSS_MB")
        fi
        ;;

    coverage)
        ARGS=()
        ARGS+=("$FUZZ_CORPUS_DIR")
        ARGS+=("-runs=0")
        mkdir -p "$FUZZ_OUTPUT_DIR/coverage" 2>/dev/null || true
        export LLVM_PROFILE_FILE="$FUZZ_OUTPUT_DIR/coverage/default.profraw"
        ;;

    minimize)
        MERGE_DIR=$(mktemp -d)
        ARGS=("-merge=1" "$MERGE_DIR" "$FUZZ_CORPUS_DIR")
        ;;

    reproduce)
        # Always enable triage on reproduce — dumps SSA pass trace to stderr
        # so the platform captures it in the job log alongside the crash.
        export TRIAGE="${TRIAGE:-FULL}"
        if [ -n "$FUZZ_CRASH_FILE" ] && [ -f "$FUZZ_CRASH_FILE" ]; then
            CRASH_FILE="$FUZZ_CRASH_FILE"
        else
            SEARCH_DIR="${FUZZ_REPRODUCE_DIR:-$FUZZ_CRASH_DIR}"
            CRASH_FILE=$(find "$SEARCH_DIR" -type f | head -1)
        fi
        if [ -z "$CRASH_FILE" ]; then
            echo "ERROR: No crash file found for reproduce" >&2
            exit 2
        fi
        ARGS=("$CRASH_FILE")
        ;;

    regress)
        ARGS=()
        ARGS+=("$FUZZ_CORPUS_DIR")
        ARGS+=("-runs=0")
        ARGS+=("-artifact_prefix=$FUZZ_CRASH_DIR/")
        ;;
esac

# ── Execute ────────────────────────────────────────────────────────────
# libFuzzer with -jobs=N writes fuzz-{N}.log to the current working directory.
# Must cd to a writable location on a read-only rootfs.
cd "$FUZZ_OUTPUT_DIR"

set +e
"$BINARY" "${ARGS[@]}"
EXIT_CODE=$?
set -e

# ── Post-processing ───────────────────────────────────────────────────

# Minimize: swap merged corpus back.
if [ "$FUZZ_MODE" = "minimize" ] && [ -d "${MERGE_DIR:-}" ]; then
    rm -rf "${FUZZ_CORPUS_DIR:?}/"*
    mv "$MERGE_DIR"/* "$FUZZ_CORPUS_DIR/" 2>/dev/null || true
    rm -rf "$MERGE_DIR"
fi

# Coverage: profraw → profdata → coverage.json + HTML report.
# Platform ingests /output/coverage/coverage.json (coverage/service.py:80).
if [ "$FUZZ_MODE" = "coverage" ] && [ -f "$FUZZ_OUTPUT_DIR/coverage/default.profraw" ]; then
    echo "Merging coverage data..."
    llvm-profdata-18 merge -sparse "$FUZZ_OUTPUT_DIR/coverage/default.profraw" \
        -o "$FUZZ_OUTPUT_DIR/coverage/fuzz.profdata"

    echo "Generating coverage.json (platform ingestion)..."
    llvm-cov-18 export "$BINARY" \
        -instr-profile="$FUZZ_OUTPUT_DIR/coverage/fuzz.profdata" \
        -summary-only \
        > "$FUZZ_OUTPUT_DIR/coverage/coverage.json"

    echo "Generating coverage.lcov (per-file line-level data)..."
    llvm-cov-18 export "$BINARY" \
        -instr-profile="$FUZZ_OUTPUT_DIR/coverage/fuzz.profdata" \
        -format=lcov \
        > "$FUZZ_OUTPUT_DIR/coverage/coverage.lcov"

    echo "Generating HTML coverage report..."
    llvm-cov-18 show "$BINARY" \
        -instr-profile="$FUZZ_OUTPUT_DIR/coverage/fuzz.profdata" \
        -format=html \
        -output-dir="$FUZZ_OUTPUT_DIR/coverage/cov-html" \
        -show-line-counts-or-regions \
        --show-branches=percent \
        --show-directory-coverage

    echo "Generating content hashes for cross-fuzzer dedup..."
    _hash_tmp=$(mktemp)
    grep '^SF:' "$FUZZ_OUTPUT_DIR/coverage/coverage.lcov" | cut -d: -f2- | sort -u | while IFS= read -r fpath; do
        if [ -f "$fpath" ]; then
            hash=$(sha256sum "$fpath" | cut -d' ' -f1)
            printf '"%s":"%s"\n' "$fpath" "$hash" >> "$_hash_tmp"
        fi
    done
    printf '{%s}' "$(paste -sd, "$_hash_tmp")" > "$FUZZ_OUTPUT_DIR/coverage/coverage_hashes.json"
    rm -f "$_hash_tmp"

    echo "Coverage artifacts written to $FUZZ_OUTPUT_DIR/coverage/"
fi

# libFuzzer exit codes:
#   0  = clean (timeout reached, no crash)
#   1  = crash found (sanitizer/panic)
#   77 = libFuzzer-detected issue (OOM, leak, timeout)
case $EXIT_CODE in
    0)   exit 0 ;;
    77)  exit 137 ;;
    *)   exit $EXIT_CODE ;;
esac
