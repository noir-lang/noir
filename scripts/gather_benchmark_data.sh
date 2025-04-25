#!/bin/bash

set -ue

NARGO=${NARGO:-nargo}
OUTPUT_DIR=${OUTPUT_DIR:-$(realpath "$(dirname "$0")/output")}
mkdir -p $OUTPUT_DIR

echo "PROJECT_TAG: ${PROJECT_TAG}"
echo "PROJECT_DIR: ${PROJECT_DIR}"

setup_repo() {
    local repo_slug=$1
    local repo_tag=$2
    local temp_dir=$3

    local repo_url="https://github.com/$repo_slug"
    
    # Sadly we cannot use depth=1 clones here as we need to be able to checkout
    # commit hashes as well as branches/releases
    git clone $repo_url $temp_dir
    git -C $TMP_DIR -c advice.detachedHead=false checkout $TAG
}

compile_project() {
    for ((i = 1; i <= NUM_RUNS; i++)); do
      NOIR_LOG=trace NARGO_LOG_DIR=./tmp $NARGO compile --force --silence-warnings 2>> /dev/null
    done

    mv ./tmp/* $OUTPUT_DIR/compilation.jsonl
}

execute_project() {
    for ((i = 1; i <= NUM_RUNS; i++)); do
      NOIR_LOG=trace NARGO_LOG_DIR=./tmp $NARGO execute --silence-warnings >> /dev/null
    done

    mv ./tmp/* $OUTPUT_DIR/execution.jsonl
}

save_artifact() {
    mv ./target/*.json $OUTPUT_DIR/artifact.json
}

if [ -z "${CI:-}" ]; then
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT
    
    setup_repo $REPO_SLUG $PROJECT_TAG $TMP_DIR
fi

REPO_DIR=${REPO_DIR:-$TMP_DIR}
cd "$REPO_DIR/$PROJECT_DIR"

# We run `nargo check` to pre-fetch any dependencies so we don't measure the time to download these
# when benchmarking.
$NARGO check --silence-warnings

compile_project
execute_project
save_artifact
