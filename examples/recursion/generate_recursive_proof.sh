#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

# Execute and prove inner circuit (sum)
mkdir -p ./target/sum
nargo execute sum_witness --package sum
$BACKEND prove -b ./target/sum.json -w ./target/sum_witness.gz --init_kzg_accumulator --output_format bytes_and_fields -o ./target/sum

# Generate vk for inner circuit
$BACKEND write_vk -b ./target/sum.json -o ./target/sum --init_kzg_accumulator --output_format fields

# Prepare Prover.toml for recurse_leaf
PROOF_AS_FIELDS=$(jq -r '.[0:]' ./target/sum/proof_fields.json)
PUBLIC_INPUTS_AS_FIELDS=$(jq -r '.[0:]' ./target/sum/public_inputs_fields.json)
VK_AS_FIELDS=$(jq -r '.[0:]' ./target/sum/vk_fields.json)
VK_HASH="0x0"

RECURSE_LEAF_PROVER_TOML=./recurse_leaf/Prover.toml
echo -n "" > $RECURSE_LEAF_PROVER_TOML
echo "num = 2" > $RECURSE_LEAF_PROVER_TOML
echo "key_hash = $VK_HASH" >> $RECURSE_LEAF_PROVER_TOML
echo "verification_key = $VK_AS_FIELDS"  >> $RECURSE_LEAF_PROVER_TOML
echo "proof = $PROOF_AS_FIELDS" >> $RECURSE_LEAF_PROVER_TOML
echo "public_inputs = $PUBLIC_INPUTS_AS_FIELDS" >> $RECURSE_LEAF_PROVER_TOML


# We can now execute and prove `recurse_leaf`
nargo execute recurse_leaf_witness --package recurse_leaf

mkdir -p ./target/leaf
$BACKEND prove -b ./target/recurse_leaf.json -w ./target/recurse_leaf_witness.gz --output_format bytes_and_fields --init_kzg_accumulator -o ./target/leaf
$BACKEND write_vk -b ./target/recurse_leaf.json --output_format bytes_and_fields --init_kzg_accumulator -o ./target/leaf

# Sanity check
$BACKEND verify -k ./target/leaf/vk -p ./target/leaf/proof  -i ./target/leaf/public_inputs

# Now we generate the final `recurse_node` proof similarly to how we did for `recurse_leaf`.
PROOF_AS_FIELDS=$(jq -r '.[0:]' ./target/leaf/proof_fields.json)
PUBLIC_INPUTS_AS_FIELDS=$(jq -r '.[0:]' ./target/leaf/public_inputs_fields.json)
VK_AS_FIELDS=$(jq -r '.[0:]' ./target/leaf/vk_fields.json)

RECURSE_NODE_PROVER_TOML=./recurse_node/Prover.toml
echo -n "" > $RECURSE_NODE_PROVER_TOML
echo "key_hash = $VK_HASH" >> $RECURSE_NODE_PROVER_TOML
echo "verification_key = $VK_AS_FIELDS"  >> $RECURSE_NODE_PROVER_TOML
echo "proof = $PROOF_AS_FIELDS" >> $RECURSE_NODE_PROVER_TOML
echo "public_inputs = $PUBLIC_INPUTS_AS_FIELDS" >> $RECURSE_NODE_PROVER_TOML

# We can now execute and prove `recurse_node`

nargo execute recurse_node_witness --package recurse_node

mkdir -p ./target/node
$BACKEND prove -b ./target/recurse_node.json -w ./target/recurse_node_witness.gz -o ./target/node --init_kzg_accumulator
$BACKEND write_vk -b ./target/recurse_node.json -o ./target/node --init_kzg_accumulator

$BACKEND verify -k ./target/node/vk -p ./target/node/proof  -i ./target/node/public_inputs
