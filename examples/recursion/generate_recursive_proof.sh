#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo execute sum_witness --package sum

# Once we have generated our inner proof, we must use this to generate inputs to `recurse_leaf``
$BACKEND write_vk -b ./target/sum.json -o ./target --init_kzg_accumulator --output_format fields
mv ./target/vk_fields.json ./target/sum_vk_as_fields

VK_AS_FIELDS=$(jq -r '.[0:]' ./target/sum_vk_as_fields)
VK_HASH="0x0"

FULL_PROOF_AS_FIELDS="$($BACKEND prove -b ./target/sum.json -w ./target/sum_witness.gz --init_kzg_accumulator --output_format fields -o -)"

echo $FULL_PROOF_AS_FIELDS | jq 'length'
# sum has 3 public inputs
PUBLIC_INPUTS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[3:6]')
PROOF_AS_FIELDS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[:3] + .[6:]')

RECURSE_LEAF_PROVER_TOML=./recurse_leaf/Prover.toml
echo -n "" > $RECURSE_LEAF_PROVER_TOML
echo "num = 2" > $RECURSE_LEAF_PROVER_TOML
echo "key_hash = $VK_HASH" >> $RECURSE_LEAF_PROVER_TOML
echo "verification_key = $VK_AS_FIELDS"  >> $RECURSE_LEAF_PROVER_TOML
echo "proof = $PROOF_AS_FIELDS" >> $RECURSE_LEAF_PROVER_TOML
echo "public_inputs = $PUBLIC_INPUTS" >> $RECURSE_LEAF_PROVER_TOML

# We can now execute and prove `recurse_leaf`

nargo execute recurse_leaf_witness --package recurse_leaf

$BACKEND prove -b ./target/recurse_leaf.json -w ./target/recurse_leaf_witness.gz -o ./target --init_kzg_accumulator
mv ./target/proof ./target/recurse_leaf_proof

# Let's do a sanity check that the proof we've generated so far is valid.
$BACKEND write_vk -b ./target/recurse_leaf.json -o ./target --init_kzg_accumulator
mv ./target/vk ./target/recurse_leaf_key
$BACKEND verify -p ./target/recurse_leaf_proof -k ./target/recurse_leaf_key

# Now we generate the final `recurse_node` proof similarly to how we did for `recurse_leaf`.
$BACKEND write_vk -b ./target/recurse_leaf.json --output_format fields -o ./target --init_kzg_accumulator
mv ./target/vk_fields.json ./target/recurse_leaf_vk_as_fields
VK_AS_FIELDS=$(jq -r '.[0:]' ./target/recurse_leaf_vk_as_fields)

FULL_PROOF_AS_FIELDS="$($BACKEND prove -b ./target/recurse_leaf.json -w ./target/recurse_leaf_witness.gz --honk_recursion 1 --init_kzg_accumulator --output_format fields -o -)"
# recurse_leaf has 4 public inputs (excluding aggregation object)
PUBLIC_INPUTS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[3:7]')
PROOF_AS_FIELDS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[:3] + .[7:]')

RECURSE_NODE_PROVER_TOML=./recurse_node/Prover.toml
echo -n "" > $RECURSE_NODE_PROVER_TOML
echo "key_hash = $VK_HASH" >> $RECURSE_NODE_PROVER_TOML
echo "verification_key = $VK_AS_FIELDS"  >> $RECURSE_NODE_PROVER_TOML
echo "proof = $PROOF_AS_FIELDS" >> $RECURSE_NODE_PROVER_TOML
echo "public_inputs = $PUBLIC_INPUTS" >> $RECURSE_NODE_PROVER_TOML

# We can now execute and prove `recurse_node`

nargo execute recurse_node_witness --package recurse_node
$BACKEND prove -b ./target/recurse_node.json -w ./target/recurse_node_witness.gz -o ./target --honk_recursion 1 --init_kzg_accumulator
mv ./target/proof ./target/recurse_node_proof

# We finally verify that the generated recursive proof is valid.
$BACKEND write_vk -b ./target/recurse_node.json -o ./target --honk_recursion 1 --init_kzg_accumulator
mv ./target/vk ./target/recurse_node_key

$BACKEND verify -p ./target/recurse_node_proof -k ./target/recurse_node_key
