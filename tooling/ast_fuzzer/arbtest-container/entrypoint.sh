#!/usr/bin/env bash
set -euo pipefail

# Platform runs as UID 65534:GID 10001 — ensure files written to bind mounts
# (/crashes, /corpus, /output) are group-accessible for the worker process.
umask 000

# ContFuzzer AST fuzzer entrypoint (arbtest variant).
#
# Runs the pre-built test binary directly — no cargo at runtime.
# The binary is built during docker build and copied to /targets/fuzzing/.
#
# Each target (e.g. comptime_vs_brillig_direct) is a #[test] function
# inside the binary. The test name is passed as a substring filter.
#
# Platform contract (FUZZ_* env vars):
#   FUZZ_MODE          fuzz | reproduce | regress
#   FUZZ_TARGET        target name (e.g. comptime_vs_brillig_direct)
#   FUZZ_CORPUS_DIR    /corpus (regress: seed files to replay)
#   FUZZ_CRASH_DIR     /crashes
#   FUZZ_OUTPUT_DIR    /output
#   FUZZ_TIMEOUT       total seconds for the job (outer loop budget)
#   FUZZ_REPRODUCE_DIR /reproduce-input (reproduce mode only)
#   FUZZ_CRASH_FILE    path to seed file (reproduce mode only)
#
# Unused (no coverage in arbtest mode):
#   FUZZ_JOBS, FUZZ_MEMORY

: "${FUZZ_MODE:=fuzz}"
: "${FUZZ_TARGET:=}"
: "${FUZZ_CORPUS_DIR:=/corpus}"
: "${FUZZ_CRASH_DIR:=/crashes}"
: "${FUZZ_OUTPUT_DIR:=/output}"
: "${FUZZ_TIMEOUT:=0}"
: "${FUZZ_REPRODUCE_DIR:=}"
: "${FUZZ_CRASH_FILE:=}"

# Per-iteration budget for arbtest (seconds). Each test run fuzzes
# for this long, then exits 0 if no crash. The outer loop restarts it
# until FUZZ_TIMEOUT is reached or a crash is found.
: "${ARBTEST_BUDGET:=60}"

TEST_BINARY="/targets/fuzzing/noir_ast_fuzzer_fuzz"

if [ -z "$FUZZ_TARGET" ]; then
    echo "ERROR: FUZZ_TARGET is required" >&2
    exit 2
fi

if [ ! -x "$TEST_BINARY" ]; then
    echo "ERROR: Test binary not found: $TEST_BINARY" >&2
    exit 2
fi

mkdir -p "$FUZZ_CORPUS_DIR" "$FUZZ_CRASH_DIR" "$FUZZ_OUTPUT_DIR" 2>/dev/null || true

# Strip ANSI escape codes from a string.
_strip_ansi() {
    sed 's/\x1b\[[0-9;]*m//g'
}

# Extract the arbtest seed from test output.
# arbtest prints: "    Seed: \x1b[1m0x<16 hex>\x1b[0m"
_extract_seed() {
    _strip_ansi | grep -oP 'Seed:\s*\K0x[0-9a-fA-F]+' | head -1
}

# Run the test binary with a specific target filter.
# The binary is a Rust test harness — target name is a substring filter.
_run_test() {
    "$TEST_BINARY" "$FUZZ_TARGET" --test-threads=1 "$@"
}

# ── Reproduce mode ────────────────────────────────────────────────────
if [ "$FUZZ_MODE" = "reproduce" ]; then
    # The platform mounts a seed file. Read the seed hex from it.
    if [ -n "$FUZZ_CRASH_FILE" ] && [ -f "$FUZZ_CRASH_FILE" ]; then
        SEED_FILE="$FUZZ_CRASH_FILE"
    else
        SEARCH_DIR="${FUZZ_REPRODUCE_DIR:-$FUZZ_CRASH_DIR}"
        SEED_FILE=$(find "$SEARCH_DIR" -type f | head -1)
    fi
    if [ -z "$SEED_FILE" ]; then
        echo "ERROR: No seed file found for reproduce" >&2
        exit 2
    fi

    SEED=$(cat "$SEED_FILE" | _strip_ansi | grep -oP '0x[0-9a-fA-F]+' | head -1)
    if [ -z "$SEED" ]; then
        SEED=$(cat "$SEED_FILE" | tr -d '[:space:]')
    fi
    if [ -z "$SEED" ]; then
        echo "ERROR: Could not extract seed from $SEED_FILE" >&2
        exit 2
    fi

    echo "Reproducing with seed: $SEED"
    export NOIR_AST_FUZZER_SEED="$SEED"
    export RUST_BACKTRACE=1

    set +e
    _run_test 2>&1 | tee "$FUZZ_OUTPUT_DIR/reproduce.log"
    EXIT_CODE=${PIPESTATUS[0]}
    set -e

    if [ $EXIT_CODE -ne 0 ]; then
        echo "Crash reproduced (seed=$SEED, exit=$EXIT_CODE)"
        exit 1
    else
        echo "Seed $SEED did not reproduce a crash"
        exit 0
    fi
fi

# ── Regress mode ──────────────────────────────────────────────────────
if [ "$FUZZ_MODE" = "regress" ]; then
    export RUST_BACKTRACE=1
    export RUST_MIN_STACK="${RUST_MIN_STACK:-8388608}"

    SEED_FILES=$(find "$FUZZ_CORPUS_DIR" -type f 2>/dev/null | sort)
    if [ -z "$SEED_FILES" ]; then
        echo "No seed files found in $FUZZ_CORPUS_DIR — nothing to regress"
        exit 0
    fi

    TOTAL=$(echo "$SEED_FILES" | wc -l | tr -d ' ')
    CURRENT=0
    FAILURES=0

    echo "Regressing $TOTAL seed(s)..."

    while IFS= read -r SEED_FILE; do
        CURRENT=$((CURRENT + 1))
        SEED=$(cat "$SEED_FILE" | _strip_ansi | grep -oP '0x[0-9a-fA-F]+' | head -1)
        if [ -z "$SEED" ]; then
            SEED=$(cat "$SEED_FILE" | tr -d '[:space:]')
        fi
        if [ -z "$SEED" ]; then
            echo "[$CURRENT/$TOTAL] SKIP $(basename "$SEED_FILE") — no seed found"
            continue
        fi

        echo "[$CURRENT/$TOTAL] Testing seed $SEED..."
        export NOIR_AST_FUZZER_SEED="$SEED"

        # Write directly to the sidecar path. Shell redirection respects
        # umask (unlike mktemp which forces 0600), so the file is created
        # world-readable. If the seed passes, we delete it.
        SIDECAR="$FUZZ_OUTPUT_DIR/seed-${SEED#0x}_result.txt"
        set +e
        _run_test 2>&1 | tee "$SIDECAR"
        EXIT_CODE=${PIPESTATUS[0]}
        set -e

        if [ $EXIT_CODE -ne 0 ]; then
            FAILURES=$((FAILURES + 1))
            echo "[$CURRENT/$TOTAL] CRASH seed $SEED"
            echo "$SEED" > "$FUZZ_CRASH_DIR/seed-${SEED#0x}"
        else
            echo "[$CURRENT/$TOTAL] OK seed $SEED"
            rm -f "$SIDECAR"
        fi
    done <<< "$SEED_FILES"

    echo "Regress complete: $FAILURES/$TOTAL failed"
    if [ "$FAILURES" -gt 0 ]; then
        exit 1
    fi
    exit 0
fi

# ── Fuzz mode ─────────────────────────────────────────────────────────
if [ "$FUZZ_MODE" != "fuzz" ]; then
    echo "ERROR: Unsupported FUZZ_MODE=$FUZZ_MODE (this container supports: fuzz, reproduce, regress)" >&2
    exit 2
fi

export NOIR_AST_FUZZER_FORCE_NON_DETERMINISTIC=1
export NOIR_AST_FUZZER_BUDGET_SECS="$ARBTEST_BUDGET"
# No backtrace in fuzz mode — the comparison failure output from the fuzzer
# is what matters; Rust backtraces just add noise and bloat the 10 MiB log cap.
# export RUST_BACKTRACE=1
export RUST_MIN_STACK="${RUST_MIN_STACK:-8388608}"

START_TIME=$(date +%s)
ITERATION=0
CRASH_FOUND=0

while true; do
    ITERATION=$((ITERATION + 1))
    ELAPSED=$(( $(date +%s) - START_TIME ))

    if [ "$FUZZ_TIMEOUT" -gt 0 ] 2>/dev/null && [ "$ELAPSED" -ge "$FUZZ_TIMEOUT" ]; then
        echo "Total timeout reached after ${ELAPSED}s (${ITERATION} iterations)"
        break
    fi

    if [ "$FUZZ_TIMEOUT" -gt 0 ] 2>/dev/null; then
        REMAINING=$((FUZZ_TIMEOUT - ELAPSED))
        if [ "$REMAINING" -le 0 ]; then
            break
        fi
        if [ "$REMAINING" -lt "$ARBTEST_BUDGET" ]; then
            export NOIR_AST_FUZZER_BUDGET_SECS="$REMAINING"
        fi
    fi

    echo "=== Iteration $ITERATION (elapsed: ${ELAPSED}s) ==="

    # Tee to a per-iteration log. On success it's deleted; on crash it
    # becomes the sidecar (renamed once we know the seed).
    ITER_LOG="$FUZZ_OUTPUT_DIR/iter-${ITERATION}.log"

    set +e
    _run_test 2>&1 | tee "$ITER_LOG"
    EXIT_CODE=${PIPESTATUS[0]}
    set -e

    if [ $EXIT_CODE -ne 0 ]; then
        CRASH_FOUND=1
        echo "Crash detected in iteration $ITERATION (exit=$EXIT_CODE)"

        SEED=$(cat "$ITER_LOG" | _extract_seed)

        if [ -n "$SEED" ]; then
            echo "$SEED" > "$FUZZ_CRASH_DIR/seed-${SEED#0x}"
            echo "Seed written to $FUZZ_CRASH_DIR/seed-${SEED#0x}"

            # Rename the log to the sidecar name the platform expects.
            mv "$ITER_LOG" "$FUZZ_OUTPUT_DIR/seed-${SEED#0x}_result.txt"
            echo "Sidecar written to $FUZZ_OUTPUT_DIR/seed-${SEED#0x}_result.txt"
        else
            echo "WARN: Could not extract seed from crash output" >&2
            mv "$ITER_LOG" "$FUZZ_CRASH_DIR/crash-iter-${ITERATION}"
        fi

        break
    fi

    rm -f "$ITER_LOG"
done

if [ "$CRASH_FOUND" -eq 1 ]; then
    exit 1
else
    exit 0
fi
