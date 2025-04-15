#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

# Execute and prove inner circuit (sum)
mkdir -p ./target/sum
nargo execute sum_witness --package sum --pedantic-solving
$BACKEND prove -b ./target/sum.json -w ./target/sum_witness.gz --init_kzg_accumulator --output_format bytes_and_fields -o ./target/sum

# Generate vk for inner circuit
$BACKEND write_vk -b ./target/sum.json -o ./target/sum --init_kzg_accumulator --output_format fields


# Prepare Prover.toml for recurse_leaf
RECURSE_LEAF_PROVER_TOML=./recurse_leaf/Prover.toml
echo -n "" > $RECURSE_LEAF_PROVER_TOML
echo "num = 2" > $RECURSE_LEAF_PROVER_TOML
echo "verification_key = $(cat ./target/sum/vk_fields.json)"  >> $RECURSE_LEAF_PROVER_TOML
echo "proof = $(cat ./target/sum/proof_fields.json)" >> $RECURSE_LEAF_PROVER_TOML
echo "public_inputs = $(cat ./target/sum/public_inputs_fields.json)" >> $RECURSE_LEAF_PROVER_TOML
echo "key_hash = 0x0" >> $RECURSE_LEAF_PROVER_TOML  # VK hash is not implemented yet


# Execute and prove `recurse_leaf`
nargo execute recurse_leaf_witness --package recurse_leaf --pedantic-solving

mkdir -p ./target/leaf
$BACKEND prove -b ./target/recurse_leaf.json -w ./target/recurse_leaf_witness.gz --output_format bytes_and_fields --init_kzg_accumulator -o ./target/leaf
$BACKEND write_vk -b ./target/recurse_leaf.json --output_format bytes_and_fields --init_kzg_accumulator -o ./target/leaf

# Sanity check
$BACKEND verify -k ./target/leaf/vk -p ./target/leaf/proof  -i ./target/leaf/public_inputs


# Generate Prover.toml for `recurse_node`
RECURSE_NODE_PROVER_TOML=./recurse_node/Prover.toml
echo -n "" > $RECURSE_NODE_PROVER_TOML
echo "key_hash = 0x0" >> $RECURSE_NODE_PROVER_TOML
echo "verification_key = $(cat ./target/leaf/vk_fields.json)"  >> $RECURSE_NODE_PROVER_TOML
echo "proof = $(cat ./target/leaf/proof_fields.json)" >> $RECURSE_NODE_PROVER_TOML
echo "public_inputs = $(cat ./target/leaf/public_inputs_fields.json)" >> $RECURSE_NODE_PROVER_TOML


# Execute and prove `recurse_node`
nargo execute recurse_node_witness --package recurse_node --pedantic-solving

mkdir -p ./target/node
$BACKEND prove -b ./target/recurse_node.json -w ./target/recurse_node_witness.gz -o ./target/node --init_kzg_accumulator
$BACKEND write_vk -b ./target/recurse_node.json -o ./target/node --init_kzg_accumulator

$BACKEND verify -k ./target/node/vk -p ./target/node/proof  -i ./target/node/public_inputs
