#!/bin/bash

set -ue

NARGO=${NARGO:-nargo}
OUTPUT_DIR=$(realpath ${OUTPUT_DIR:-"$(dirname "$0")/output")})
mkdir -p $OUTPUT_DIR

echo "PROJECT_TAG: ${PROJECT_TAG}"
echo "PROJECT_DIR: ${PROJECT_DIR}"

# Silence logs from the Elaborator and other frontend stuff,
# otherwise it can take too long and produce too much data.
NOIR_LOG=trace,noirc_frontend=off

setup_repo() {
    local repo_slug=$1
    local repo_tag=$2
    local temp_dir=$3

    local repo_url="https://github.com/$repo_slug"

    # Sadly we cannot use depth=1 clones here as we need to be able to checkout
    # commit hashes as well as branches/releases
    git clone $repo_url $temp_dir
    git -C $temp_dir -c advice.detachedHead=false checkout $repo_tag
}

# The callers of `run_nargo` silence its output, so warnings go to the script's own stdout.
exec 3>&1

# Runs `nargo` with the given arguments, logging to `./tmp`. The first argument is the run number.
#
# When `NARGO_BASELINE` is set, the baseline binary runs too, logging to `./tmp_baseline` and with
# its own target directory. Both binaries are then timed on the same machine, so their ratio doesn't
# depend on which runner the job landed on. The baseline goes first on odd runs and second on even
# ones; the Brillig phases start counting at 2, so a phase with a single run doesn't always put the
# same binary first.
run_nargo() {
    local run=$1
    shift
    if [ -z "${NARGO_BASELINE:-}" ]; then
      NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp $NARGO "$@"
    elif ((run % 2 == 1)); then
      run_baseline "$@"
      NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp $NARGO "$@"
    else
      NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp $NARGO "$@"
      run_baseline "$@"
    fi
}

# Runs the baseline binary. It may fail where the PR's doesn't, for example on a project that needs
# the PR's compiler, so a failure drops the baseline for the rest of the project rather than the
# PR's own results. Phases saved before then keep their baseline logs.
run_baseline() {
    if ! NOIR_LOG=$NOIR_LOG NARGO_LOG_DIR=./tmp_baseline $NARGO_BASELINE --target-dir ./target_baseline "$@"; then
      echo "::warning::The baseline nargo failed on \`nargo $*\`; reporting this project without it from here on" >&3
      unset NARGO_BASELINE
      rm -rf ./tmp_baseline
    fi
}

save_logs() {
    local name=$1
    mv ./tmp/* $OUTPUT_DIR/$name.jsonl
    if [ -n "${NARGO_BASELINE:-}" ]; then
      mv ./tmp_baseline/* $OUTPUT_DIR/baseline_$name.jsonl
    fi
}

compile_project() {
    echo "Compiling program (ACIR)"
    for ((i = 1; i <= NUM_COMPILE_RUNS; i++)); do
      run_nargo $i compile --force --silence-warnings 2>> /dev/null
    done

    save_logs compilation
}

execute_project() {
    echo "Executing program (ACIR)"
    for ((i = 1; i <= NUM_EXECUTE_RUNS; i++)); do
      run_nargo $i execute --silence-warnings >> /dev/null
    done

    save_logs execution
}

save_artifact() {
    echo "Copying artifact (ACIR)"
    mv ./target/*.json $OUTPUT_DIR/artifact.json
}

compile_brillig_project() {
    echo "Compiling program (Brillig)"
    for ((i = 1; i <= NUM_COMPILE_RUNS; i++)); do
      run_nargo $((i + 1)) compile --force --force-brillig --silence-warnings 2>> /dev/null
    done

    save_logs brillig_compilation
}

execute_brillig_project() {
    echo "Executing program (Brillig)"
    for ((i = 1; i <= NUM_EXECUTE_RUNS; i++)); do
      run_nargo $((i + 1)) execute --force-brillig --silence-warnings >> /dev/null
    done

    save_logs brillig_execution
}

save_brillig_artifact() {
    echo "Copying artifact (Brillig)"
    mv ./target/*.json $OUTPUT_DIR/brillig_artifact.json
}

if [ -z "${REPO_DIR:-}" ]; then
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    setup_repo $REPO_SLUG $PROJECT_TAG $TMP_DIR
fi

REPO_DIR=${REPO_DIR:-$TMP_DIR}
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
