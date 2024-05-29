#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

./clone_test_vectors.sh

TEST_NAMES=("$@")
THREADS=(1 4 16 32 64)
BENCHMARKS=$LOG_FILE

if [[ -z "${LOG_FILE}" ]]; then
    BENCHMARKS=$(mktemp)
fi

if [ "${#TEST_NAMES[@]}" -eq 0 ]; then
    TEST_NAMES=$(find acir_tests/bench_* -maxdepth 0 -type d -printf '%f ')
fi

for TEST in ${TEST_NAMES[@]}; do
    for HC in ${THREADS[@]}; do
        HARDWARE_CONCURRENCY=$HC BENCHMARK_FD=3 ./run_acir_tests.sh $TEST 3>>$BENCHMARKS
    done
done

# Build results into string with \n delimited rows and space delimited values.
TABLE_DATA=""
for TEST in ${TEST_NAMES[@]}; do
    GATE_COUNT=$(jq -r --arg test "$TEST" 'select(.eventName == "gate_count" and .acir_test == $test) | .value' $BENCHMARKS | uniq)
    SUBGROUP_SIZE=$(jq -r --arg test "$TEST" 'select(.eventName == "subgroup_size" and .acir_test == $test) | .value' $BENCHMARKS | uniq)
    # Name in col 1, gate count in col 2, subgroup size in col 3.
    TABLE_DATA+="$TEST $GATE_COUNT $SUBGROUP_SIZE"
    # Each thread timing in subsequent cols.
    for HC in "${THREADS[@]}"; do
        RESULT=$(cat $BENCHMARKS | jq -r --arg test "$TEST" --argjson hc $HC 'select(.eventName == "proof_construction_time" and .acir_test == $test and .threads == $hc) | .value')
        TABLE_DATA+=" $RESULT"
    done
    TABLE_DATA+=$'\n'
done

# Trim the trailing newline.
TABLE_DATA="${TABLE_DATA%$'\n'}"

echo
echo Table represents time in ms to build circuit and proof for each test on n threads.
echo Ignores proving key construction.
echo
# Use awk to print the table
echo -e "$TABLE_DATA" | awk -v threads="${THREADS[*]}" 'BEGIN {
    split(threads, t, " ");
    len_threads = length(t);
    print "+--------------------------+------------+---------------+" genseparator(len_threads);
    print "| Test                     | Gate Count | Subgroup Size |" genthreadheaders(t, len_threads);
    print "+--------------------------+------------+---------------+" genseparator(len_threads);
}
{
    printf("| %-24s | %-10s | %-13s |", $1, $2, $3);
    for (i = 4; i <= len_threads+3; i++) {
        printf " %9s |", $(i);
    }
    print "";
}
END {
    print "+--------------------------+------------+---------------+" genseparator(len_threads);
}
function genseparator(len,   res) {
    for (i = 1; i <= len; i++) res = res "-----------+";
    return res;
}
function genthreadheaders(t, len,   res) {
    for (i = 1; i <= len; i++) res = res sprintf(" %9s |", t[i]);
    return res;
}
'

if [[ -z "${LOG_FILE}" ]]; then
    rm $BENCHMARKS
fi
