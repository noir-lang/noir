#!/bin/sh
set -eu

export PROOF="$(pwd)/proof"
export PROOF_AS_FIELDS="$(pwd)/proof_fields.json"

# Create a proof, write the solidity contract, write the proof as fields in order to extract the public inputs
$BIN prove_keccak_ultra_honk -o proof
$BIN write_vk_ultra_honk -o vk
$BIN proof_as_fields_honk -k vk -c $CRS_PATH -p $PROOF
$BIN contract_ultra_honk -k vk -c $CRS_PATH -b ./target/program.json -o Verifier.sol

# Export the paths to the environment variables for the js test runner
export VERIFIER_PATH="$(pwd)/Verifier.sol"
export TEST_PATH=$(realpath "../../sol-test/HonkTest.sol")
export TESTING_HONK="true"

# Use solcjs to compile the generated key contract with the template verifier and test contract 
# index.js will start an anvil, on a random port
# Deploy the verifier then send a test transaction
export TEST_NAME=$(basename $(pwd))
node ../../sol-test/src/index.js
