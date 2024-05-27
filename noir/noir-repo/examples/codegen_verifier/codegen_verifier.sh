#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo compile

# TODO: backend should automatically generate vk if necessary.
$BACKEND write_vk -b ./target/hello_world.json
$BACKEND contract -o ./src/contract.sol

# We now generate a proof and check whether the verifier contract will verify it.

nargo execute witness

PROOF_PATH=./target/proof
$BACKEND prove -b ./target/hello_world.json -w ./target/witness.gz -o $PROOF_PATH

NUM_PUBLIC_INPUTS=1
PUBLIC_INPUT_BYTES=$((32 * $NUM_PUBLIC_INPUTS))
HEX_PUBLIC_INPUTS=$(head -c $PUBLIC_INPUT_BYTES $PROOF_PATH | od -An -v -t x1 | tr -d $' \n')
HEX_PROOF=$(tail -c +$(($PUBLIC_INPUT_BYTES + 1)) $PROOF_PATH | od -An -v -t x1 | tr -d $' \n')

# Spin up an anvil node to deploy the contract to
anvil &

DEPLOY_INFO=$(forge create UltraVerifier \
    --rpc-url "127.0.0.1:8545" \
    --private-key "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" \
    --json)
VERIFIER_ADDRESS=$(echo $DEPLOY_INFO | jq -r '.deployedTo')

# Call the verifier contract with our proof.
# Note that we haven't needed to split up `HEX_PUBLIC_INPUTS` as there's only a single public input
cast call $VERIFIER_ADDRESS "verify(bytes, bytes32[])(bool)" "0x$HEX_PROOF" "[0x$HEX_PUBLIC_INPUTS]"

# Stop anvil node again
kill %-