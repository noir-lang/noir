#!/usr/bin/env bash
set -eu

OUT=${1:-bytecode-size.jsonl}
rm -f $OUT

for file in aztec-packages/noir-projects/noir-protocol-circuits/target/*.json; do
    PROGRAM=$(basename $file .json)
    cat $file \
        | jq --arg PROGRAM $PROGRAM \
            -c '{name: $PROGRAM, bytecode_size: .bytecode | @base64d | length}' \
        >> $OUT
done

for file in aztec-packages/noir-projects/noir-contracts/target/*.json; do
    CONTRACT=$(basename $file .json)
    cat $file \
        | jq --arg CONTRACT $CONTRACT \
            -c '.functions | sort_by(.name) | .[] | {name: ($CONTRACT + "::" + .name), "bytecode_size": .bytecode | @base64d | length}' \
        >> $OUT
done
