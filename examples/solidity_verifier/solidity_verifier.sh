#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}
BIN_TO_FLDS=../binary_to_fields.py

nargo compile

$BACKEND write_vk -b ./target/hello_world.json -o ./target --oracle_hash keccak
$BACKEND write_solidity_verifier -k ./target/vk -o ./src/contract.sol

# We now generate a proof and check whether the verifier contract will verify it.
nargo execute --pedantic-solving witness

# Generate proof
$BACKEND prove -b ./target/hello_world.json -w ./target/witness.gz --oracle_hash keccak -o ./target

# Sanity check that proof is valid.
$BACKEND verify -k ./target/vk -p ./target/proof -i ./target/public_inputs --oracle_hash keccak

# Read proof and convert to hex string
PROOF_HEX=$(cat ./target/proof | od -An -v -t x1 | tr -d $' \n')
# $BIN_TO_FLDS prints each public input in hex format, but we need to remove quotes for using in `cast`
PUBLIC_INPUTS_HEX=$(python3 $BIN_TO_FLDS ./target/public_inputs | tr -d '"')

# Spin up an anvil node to deploy the contract to
#
# Code size limit is set to 10x normal to avoid being broken in case contracts
# are too large. Recommended to remove in your code.
anvil --code-size-limit=400000 &
trap 'kill %-' EXIT

DEPLOY_INFO=$(forge create HonkVerifier \
  --rpc-url "127.0.0.1:8545" \
  --private-key "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" \
  --broadcast \
  --json)
VERIFIER_ADDRESS=$(echo $DEPLOY_INFO | jq -r '.deployedTo')

# Call the verifier contract with our proof.
cast call "$VERIFIER_ADDRESS" "verify(bytes, bytes32[])(bool)" "$PROOF_HEX" "$PUBLIC_INPUTS_HEX"
