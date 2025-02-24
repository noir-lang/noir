#!/bin/bash

AZTEC_COMMIT=$(git ls-remote https://github.com/AztecProtocol/aztec-packages.git HEAD | grep -oE '^\b[0-9a-f]{40}\b')

function bump_commit() {
    FILE=$1
    AZTEC_COMMIT=$AZTEC_COMMIT yq -i '.define = env(AZTEC_COMMIT)' $FILE

}

bump_commit ./EXTERNAL_NOIR_LIBRARIES.yml
bump_commit ./.github/benchmark_projects.yml
