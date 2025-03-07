#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo execute --pedantic-solving witness

# TODO: `bb` should create `proofs` directory if it doesn't exist.
mkdir -p proofs
$BACKEND OLD_API prove -b ./target/hello_world.json -w ./target/witness.gz

# TODO: backend should automatically generate vk if necessary.
$BACKEND OLD_API write_vk -b ./target/hello_world.json
$BACKEND OLD_API verify -k ./target/vk -p ./proofs/proof
