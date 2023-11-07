#!/bin/sh
set -eu

export PROOF="$(pwd)/proof"
export PROOF_AS_FIELDS="$(pwd)/proof_fields.json"

# Create a proof, write the solidity contract, write the proof as fields in order to extract the public inputs
$BIN prove -o proof
$BIN write_vk  -o vk
$BIN proof_as_fields -k vk -c $CRS_PATH -p $PROOF
$BIN contract -k vk -c $CRS_PATH -b ./target/acir.gz -o Key.sol

# Export the paths to the environment variables for the js test runner
export KEY_PATH="$(pwd)/Key.sol"
export VERIFIER_PATH=$(realpath "../../sol-test/Verifier.sol")
export TEST_PATH=$(realpath "../../sol-test/Test.sol")
export BASE_PATH=$(realpath "../../../sol/src/ultra/BaseUltraVerifier.sol")

# Use solcjs to compile the generated key contract with the template verifier and test contract 
# index.js will start an anvil, on a random port
# Deploy the verifier then send a test transaction
export TEST_NAME=$(basename $(pwd))
node ../../sol-test/src/index.js