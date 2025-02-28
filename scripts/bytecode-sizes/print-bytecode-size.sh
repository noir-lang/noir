#!/usr/bin/env bash
set -eu

AZTEC_PACKAGES_DIR=$1

for file in $AZTEC_PACKAGES_DIR/noir-projects/noir-protocol-circuits/target/*.json; do
    PROGRAM=$(basename $file .json)
    cat $file \
        | jq --arg PROGRAM $PROGRAM \
            -c '{name: $PROGRAM, bytecode_size: .bytecode | @base64d | length}'
done

for file in $AZTEC_PACKAGES_DIR/noir-projects/noir-contracts/target/*.json; do
    CONTRACT=$(basename $file .json)
    cat $file \
        | jq --arg CONTRACT $CONTRACT \
            -c '.functions | sort_by(.name) | .[] | {name: ($CONTRACT + "::" + .name), "bytecode_size": .bytecode | @base64d | length}'
done
