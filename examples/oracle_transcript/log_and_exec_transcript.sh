#!/bin/bash
set -eu

cd $(dirname $0)

# Execute the test to capture oracle calls.
NARGO_TEST_FOREIGN_CALL_LOG=Oracle.test.jsonl nargo test

# Get rid of the mock setup calls
cat Oracle.test.jsonl \
    | jq --slurp -r -c '.[] | select(.call.function | contains("mock") | not)' \
    > Oracle.jsonl

# Execute `main` with the Prover.toml and Oracle.jsonl files.
nargo execute --skip-underconstrained-check --pedantic-solving --oracle-file Oracle.jsonl

# Also execute through `noir-execute`
noir-execute \
    execute \
    --artifact-path target/oracle_transcript.json \
    --oracle-file Oracle.jsonl \
    --prover-file Prover.toml \
    --output-dir target
