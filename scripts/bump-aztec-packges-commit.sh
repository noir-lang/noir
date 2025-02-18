#!/bin/bash

AZTEC_COMMIT=$(git ls-remote https://github.com/AztecProtocol/aztec-packages.git HEAD | grep -oE '^\b[0-9a-f]{40}\b')

function bump_commit() {
    FILE=$1
    sed -E -i.bak "s/(^define: &AZ_COMMIT) .*/\1 $AZTEC_COMMIT/" $FILE  && rm $FILE.bak

}

bump_commit ./EXTERNAL_NOIR_LIBRARIES.yml
bump_commit ./.github/benchmark_projects.yml
