#!/usr/bin/env bash
# Extract per-test timing from nargo test JSON output and optionally compare two runs.
#
# Usage:
#   ./extract-test-timing.sh <results.jsonl>                     # Show per-test times
#   ./extract-test-timing.sh <baseline.jsonl> <test.jsonl>       # Compare two runs

set -eu

if [ $# -eq 1 ]; then
    echo "Per-test timing from $1:"
    echo ""
    jq -r 'select(.type == "test" and .event == "ok") | "\(.exec_time | tostring | .[:8])s  \(.name)"' "$1" | sort -t's' -k1 -rn
    echo ""
    TOTAL=$(jq -s '[.[] | select(.type == "test" and .event == "ok") | .exec_time] | add' "$1")
    COUNT=$(jq -s '[.[] | select(.type == "test" and .event == "ok")] | length' "$1")
    echo "Total: ${TOTAL}s across $COUNT tests"
elif [ $# -eq 2 ]; then
    BASELINE="$1"
    TEST="$2"
    echo "Per-test timing comparison (baseline vs test):"
    echo ""
    # Create temp files with test name -> time mappings
    BTMP=$(mktemp)
    TTMP=$(mktemp)
    trap "rm -f $BTMP $TTMP" EXIT

    jq -r 'select(.type == "test" and .event == "ok") | "\(.name)\t\(.exec_time)"' "$BASELINE" | sort > "$BTMP"
    jq -r 'select(.type == "test" and .event == "ok") | "\(.name)\t\(.exec_time)"' "$TEST" | sort > "$TTMP"

    printf "%-60s %12s %12s %10s\n" "TEST" "BASELINE" "TEST" "CHANGE"
    printf "%-60s %12s %12s %10s\n" "----" "--------" "----" "------"

    join -t $'\t' "$BTMP" "$TTMP" | while IFS=$'\t' read -r name btime ttime; do
        change=$(python3 -c "b=$btime; t=$ttime; print(f'{((t-b)/b*100):+.1f}%' if b > 0 else 'N/A')")
        printf "%-60s %11.3fs %11.3fs %10s\n" "$name" "$btime" "$ttime" "$change"
    done | sort -t'%' -k4 -rn

    echo ""
    BTOTAL=$(jq -s '[.[] | select(.type == "test" and .event == "ok") | .exec_time] | add' "$BASELINE")
    TTOTAL=$(jq -s '[.[] | select(.type == "test" and .event == "ok") | .exec_time] | add' "$TEST")
    CHANGE=$(python3 -c "b=$BTOTAL; t=$TTOTAL; print(f'{((t-b)/b*100):+.1f}%' if b > 0 else 'N/A')")
    echo "Total: ${BTOTAL}s -> ${TTOTAL}s ($CHANGE)"
else
    echo "Usage: $0 <results.jsonl> [baseline.jsonl]"
    exit 1
fi
