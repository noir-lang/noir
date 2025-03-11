#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo compile

# TODO: backend should automatically generate vk if necessary.
$BACKEND write_vk -b ./target/hello_world.json -o ./target --oracle_hash keccak
$BACKEND write_solidity_verifier -k ./target/vk -o ./src/contract.sol

# We now generate a proof and check whether the verifier contract will verify it.
nargo execute --pedantic-solving witness

PROOF_PATH=./target/proof
$BACKEND prove -b ./target/hello_world.json -w ./target/witness.gz --oracle_hash keccak -o ./target

# Sanity check that proof is valid.
$BACKEND verify -k ./target/vk -p ./target/proof --oracle_hash keccak

# Prepare proof and public inputs for solidity verifier
PROOF_HEX=$(cat $PROOF_PATH | od -An -v -t x1 | tr -d $' \n' | sed 's/^.\{8\}//')

NUM_PUBLIC_INPUTS=2
PUBLIC_INPUT_HEX_CHARS=$((32 * $NUM_PUBLIC_INPUTS * 2)) # Each public input is 32 bytes, 2 chars per byte
PUBLIC_INPUT_OFFSET_CHARS=$((96 * 2)) # First 96 bytes are the proof header

# Extract public inputs from proof - from 96th byte to 96 + 32 * NUM_PUBLIC_INPUTS bytes
HEX_PUBLIC_INPUTS=${PROOF_HEX:$PUBLIC_INPUT_OFFSET_CHARS:$PUBLIC_INPUT_HEX_CHARS}
# Split public inputs into strings where each string represents a `bytes32`.
SPLIT_HEX_PUBLIC_INPUTS=$(sed -e 's/.\{64\}/0x&,/g' <<<$HEX_PUBLIC_INPUTS)

# Extract proof without public inputs - from 0 to 96 bytes + the part after public inputs
PROOF_WITHOUT_PUBLIC_INPUTS_START=${PROOF_HEX:0:$PUBLIC_INPUT_OFFSET_CHARS} 
PROOF_WITHOUT_PUBLIC_INPUTS_END=${PROOF_HEX:$(($PUBLIC_INPUT_OFFSET_CHARS + $PUBLIC_INPUT_HEX_CHARS))}
PROOF_WITHOUT_PUBLIC_INPUTS="${PROOF_WITHOUT_PUBLIC_INPUTS_START}${PROOF_WITHOUT_PUBLIC_INPUTS_END}"

# Spin up an anvil node to deploy the contract to
anvil &
trap 'kill %-' EXIT

DEPLOY_INFO=$(forge create HonkVerifier \
  --rpc-url "127.0.0.1:8545" \
  --private-key "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" \
  --json)
VERIFIER_ADDRESS=$(echo $DEPLOY_INFO | jq -r '.deployedTo')

# Call the verifier contract with our proof.
cast call $VERIFIER_ADDRESS "verify(bytes, bytes32[])(bool)" "$PROOF_WITHOUT_PUBLIC_INPUTS" "[$SPLIT_HEX_PUBLIC_INPUTS]"
