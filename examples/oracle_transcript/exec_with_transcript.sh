#!/bin/bash
set -eu

dir=$(dirname $0)

# Execute the test to capture oracle calls.
NARGO_TEST_FOREIGN_CALL_LOG=$dir/Oracle.test.jsonl \
    nargo --program-dir $dir test

# Get rid of the mock setup calls
cat $dir/Oracle.test.jsonl \
    | jq --slurp -r -c '.[] | select(.call.function | contains("mock") | not)' \
    > $dir/Oracle.jsonl

# TODO: Execute `main` with the Prover.toml and Oracle.jsonl files.
nargo execute --skip-underconstrained-check
