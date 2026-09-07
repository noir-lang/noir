#!/bin/bash

set -ue

NARGO=${NARGO:-nargo}
OUTPUT_DIR=$(realpath ${OUTPUT_DIR:-"$(dirname "$0")/output")})
mkdir -p $OUTPUT_DIR

echo "PROJECT_DIR: ${PROJECT_DIR}"

# Silence logs from the Elaborator and other frontend stuff,
# otherwise it can take too long and produce too much data.
NOIR_LOG=trace,noirc_frontend=off

compile_project() {
    echo "Compiling program (ACIR)"
    for ((i = 1; i <= NUM_COMPILE_RUNS; i++)); do
      NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp $NARGO compile --force --silence-warnings 2>> /dev/null
    done

    mv ./tmp/* $OUTPUT_DIR/compilation.jsonl
}

execute_project() {
    echo "Executing program (ACIR)"
    for ((i = 1; i <= NUM_EXECUTE_RUNS; i++)); do
      NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp $NARGO execute --silence-warnings >> /dev/null
    done

    mv ./tmp/* $OUTPUT_DIR/execution.jsonl
}

save_artifact() {
    echo "Copying artifact (ACIR)"
    mv ./target/*.json $OUTPUT_DIR/artifact.json
}

compile_brillig_project() {
    echo "Compiling program (Brillig)"
    for ((i = 1; i <= NUM_COMPILE_RUNS; i++)); do
      NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp $NARGO compile --force --force-brillig --silence-warnings 2>> /dev/null
    done

    mv ./tmp/* $OUTPUT_DIR/brillig_compilation.jsonl
}

execute_brillig_project() {
    echo "Executing program (Brillig)"
    for ((i = 1; i <= NUM_EXECUTE_RUNS; i++)); do
      NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp $NARGO execute --force-brillig --silence-warnings >> /dev/null
    done

    mv ./tmp/* $OUTPUT_DIR/brillig_execution.jsonl
}

save_brillig_artifact() {
    echo "Copying artifact (Brillig)"
    mv ./target/*.json $OUTPUT_DIR/brillig_artifact.json
}

REPO_DIR=${REPO_DIR:-"$(dirname "$0")/.."}
cd "$REPO_DIR/$PROJECT_DIR"

[[ -f ./Prover.toml ]] && HAS_PROVER_INPUTS=true

# We run `nargo check` to pre-fetch any dependencies so we don't measure the time to download these
# when benchmarking.
$NARGO check --silence-warnings

compile_project
if [ "${HAS_PROVER_INPUTS:-"false"}" == "true" ]; then
    execute_project
fi
save_artifact

compile_brillig_project
if [ "${HAS_PROVER_INPUTS:-"false"}" == "true" ] && [ "${SKIP_BRILLIG_EXECUTION:-"false"}" != "true" ]; then
    execute_brillig_project
fi
save_brillig_artifact

echo "Completed gathering benchmarks"
