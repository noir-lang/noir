#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo execute --pedantic-solving witness

# TODO: `bb` should create `proofs` directory if it doesn't exist.
mkdir -p proofs
$BACKEND prove -b ./target/hello_world.json -w ./target/witness.gz

# TODO: backend should automatically generate vk if necessary.
$BACKEND write_vk -b ./target/hello_world.json
$BACKEND verify -k ./target/vk -p ./proofs/proof
