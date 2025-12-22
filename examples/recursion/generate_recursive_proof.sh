#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}
BIN_TO_FLDS=../../scripts/binary_to_fields.py

# Execute and prove inner circuit (sum)
mkdir -p ./target/sum
nargo execute sum_witness --package sum --pedantic-solving
# Generate vk for inner circuit
$BACKEND write_vk -b ./target/sum.json -o ./target/sum
$BACKEND prove -b ./target/sum.json -w ./target/sum_witness.gz -o ./target/sum -k ./target/sum/vk

# Prepare Prover.toml for recurse_leaf
RECURSE_LEAF_PROVER_TOML=./recurse_leaf/Prover.toml
echo -n "" > $RECURSE_LEAF_PROVER_TOML
echo "num = 2" > $RECURSE_LEAF_PROVER_TOML
echo "verification_key = $(python3 $BIN_TO_FLDS ./target/sum/vk)"  >> $RECURSE_LEAF_PROVER_TOML
echo "proof = $(python3 $BIN_TO_FLDS ./target/sum/proof)" >> $RECURSE_LEAF_PROVER_TOML
echo "public_inputs = $(python3 $BIN_TO_FLDS ./target/sum/public_inputs)" >> $RECURSE_LEAF_PROVER_TOML
echo "key_hash = $(python3 $BIN_TO_FLDS ./target/sum/vk_hash | jq -c ".[0]")" >> $RECURSE_LEAF_PROVER_TOML

# Execute and prove `recurse_leaf`
nargo execute recurse_leaf_witness --package recurse_leaf --pedantic-solving

mkdir -p ./target/leaf
$BACKEND write_vk -b ./target/recurse_leaf.json -o ./target/leaf
$BACKEND prove -b ./target/recurse_leaf.json -w ./target/recurse_leaf_witness.gz -o ./target/leaf -k ./target/leaf/vk

# Sanity check
$BACKEND verify -k ./target/leaf/vk -p ./target/leaf/proof  -i ./target/leaf/public_inputs

# Generate Prover.toml for `recurse_node`
RECURSE_NODE_PROVER_TOML=./recurse_node/Prover.toml
echo -n "" > $RECURSE_NODE_PROVER_TOML
echo "key_hash = $(python3 $BIN_TO_FLDS ./target/leaf/vk_hash | jq -c ".[0]")" >> $RECURSE_NODE_PROVER_TOML
echo "verification_key = $(python3 $BIN_TO_FLDS ./target/leaf/vk)"  >> $RECURSE_NODE_PROVER_TOML
echo "proof = $(python3 $BIN_TO_FLDS ./target/leaf/proof)" >> $RECURSE_NODE_PROVER_TOML
echo "public_inputs = $(python3 $BIN_TO_FLDS ./target/leaf/public_inputs)" >> $RECURSE_NODE_PROVER_TOML


# Execute and prove `recurse_node`
nargo execute recurse_node_witness --package recurse_node --pedantic-solving

mkdir -p ./target/node
$BACKEND write_vk -b ./target/recurse_node.json -o ./target/node
$BACKEND prove -b ./target/recurse_node.json -w ./target/recurse_node_witness.gz -o ./target/node -k ./target/node/vk
$BACKEND verify -k ./target/node/vk -p ./target/node/proof  -i ./target/node/public_inputs
