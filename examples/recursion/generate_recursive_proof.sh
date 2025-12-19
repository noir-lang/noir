#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

# Execute and prove inner circuit (sum)
mkdir -p ./target/sum
nargo execute sum_witness --package sum --pedantic-solving
# Generate vk for inner circuit
$BACKEND write_vk -b ./target/sum.json -o ./target/sum
$BACKEND prove -b ./target/sum.json -w ./target/sum_witness.gz -o ./target/sum -k ./target/sum/vk

# Convert binary outputs to JSON format for Noir
python3 binary_to_fields.py ./target/sum/vk ./target/sum/vk_fields.json
python3 binary_to_fields.py ./target/sum/vk_hash ./target/sum/vk_hash_fields.json
python3 binary_to_fields.py ./target/sum/proof ./target/sum/proof_fields.json
python3 binary_to_fields.py ./target/sum/public_inputs ./target/sum/public_inputs_fields.json

# Prepare Prover.toml for recurse_leaf
RECURSE_LEAF_PROVER_TOML=./recurse_leaf/Prover.toml
echo -n "" > $RECURSE_LEAF_PROVER_TOML
echo "num = 2" > $RECURSE_LEAF_PROVER_TOML
echo "verification_key = $(cat ./target/sum/vk_fields.json)"  >> $RECURSE_LEAF_PROVER_TOML
echo "proof = $(cat ./target/sum/proof_fields.json)" >> $RECURSE_LEAF_PROVER_TOML
echo "public_inputs = $(cat ./target/sum/public_inputs_fields.json)" >> $RECURSE_LEAF_PROVER_TOML
echo "key_hash = $(cat ./target/sum/vk_hash_fields.json)" >> $RECURSE_LEAF_PROVER_TOML

# Execute and prove `recurse_leaf`
nargo execute recurse_leaf_witness --package recurse_leaf --pedantic-solving

mkdir -p ./target/leaf
$BACKEND write_vk -b ./target/recurse_leaf.json -o ./target/leaf
$BACKEND prove -b ./target/recurse_leaf.json -w ./target/recurse_leaf_witness.gz -o ./target/leaf -k ./target/leaf/vk

# Sanity check
$BACKEND verify -k ./target/leaf/vk -p ./target/leaf/proof  -i ./target/leaf/public_inputs

# Convert binary outputs to JSON format for Noir
python3 binary_to_fields.py ./target/leaf/vk ./target/leaf/vk_fields.json
python3 binary_to_fields.py ./target/leaf/vk_hash ./target/leaf/vk_hash_fields.json
python3 binary_to_fields.py ./target/leaf/proof ./target/leaf/proof_fields.json
python3 binary_to_fields.py ./target/leaf/public_inputs ./target/leaf/public_inputs_fields.json

# Generate Prover.toml for `recurse_node`
RECURSE_NODE_PROVER_TOML=./recurse_node/Prover.toml
echo -n "" > $RECURSE_NODE_PROVER_TOML
echo "key_hash = $(cat ./target/leaf/vk_hash_fields.json)" >> $RECURSE_NODE_PROVER_TOML
echo "verification_key = $(cat ./target/leaf/vk_fields.json)"  >> $RECURSE_NODE_PROVER_TOML
echo "proof = $(cat ./target/leaf/proof_fields.json)" >> $RECURSE_NODE_PROVER_TOML
echo "public_inputs = $(cat ./target/leaf/public_inputs_fields.json)" >> $RECURSE_NODE_PROVER_TOML


# Execute and prove `recurse_node`
nargo execute recurse_node_witness --package recurse_node --pedantic-solving

mkdir -p ./target/node
$BACKEND write_vk -b ./target/recurse_node.json -o ./target/node
$BACKEND prove -b ./target/recurse_node.json -w ./target/recurse_node_witness.gz -o ./target/node -k ./target/node/vk
$BACKEND verify -k ./target/node/vk -p ./target/node/proof  -i ./target/node/public_inputs
