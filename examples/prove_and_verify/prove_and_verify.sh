#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo execute --force witness

$BACKEND write_vk -b ./target/hello_world.json -o ./target
$BACKEND prove -b ./target/hello_world.json -w ./target/witness.gz -o ./proofs
$BACKEND verify -k ./target/vk -p ./proofs/proof -i ./proofs/public_inputs
