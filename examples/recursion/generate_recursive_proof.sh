#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo execute sum_witness --package sum
$BACKEND prove -b ./target/sum.json -w ./target/sum_witness.gz -o ./target/sum_proof --recursive

# Once we have generated our inner proof, we must use this to generate inputs to `recurse_leaf``

$BACKEND write_vk -b ./target/sum.json -o ./target/sum_key --recursive
$BACKEND vk_as_fields -k ./target/sum_key -o ./target/sum_vk_as_fields
VK_HASH=$(jq -r '.[0]' ./target/sum_vk_as_fields)
VK_AS_FIELDS=$(jq -r '.[1:]' ./target/sum_vk_as_fields)

FULL_PROOF_AS_FIELDS="$($BACKEND proof_as_fields -p ./target/sum_proof -k ./target/sum_key -o -)"
# sum has 3 public inputs
PUBLIC_INPUTS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[:3]')
PROOF_AS_FIELDS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[3:]')

RECURSE_LEAF_PROVER_TOML=./recurse_leaf/Prover.toml
echo "num = 2" > $RECURSE_LEAF_PROVER_TOML
echo "key_hash = \"$VK_HASH\"" >> $RECURSE_LEAF_PROVER_TOML
echo "verification_key = $VK_AS_FIELDS"  >> $RECURSE_LEAF_PROVER_TOML
echo "proof = $PROOF_AS_FIELDS" >> $RECURSE_LEAF_PROVER_TOML
echo "public_inputs = $PUBLIC_INPUTS" >> $RECURSE_LEAF_PROVER_TOML

# We can now execute and prove `recurse_leaf`

nargo execute recurse_leaf_witness --package recurse_leaf
$BACKEND prove -b ./target/recurse_leaf.json -w ./target/recurse_leaf_witness.gz -o ./target/recurse_leaf_proof --recursive

# Let's do a sanity check that the proof we've generated so far is valid.
$BACKEND write_vk -b ./target/recurse_leaf.json -o ./target/recurse_leaf_key --recursive
$BACKEND verify -p ./target/recurse_leaf_proof -k ./target/recurse_leaf_key

# Now we generate the final `recurse_node` proof similarly to how we did for `recurse_leaf`.

$BACKEND vk_as_fields -k ./target/recurse_leaf_key -o ./target/recurse_leaf_vk_as_fields
VK_HASH=$(jq -r '.[0]' ./target/recurse_leaf_vk_as_fields)
VK_AS_FIELDS=$(jq -r '.[1:]' ./target/recurse_leaf_vk_as_fields)

FULL_PROOF_AS_FIELDS="$($BACKEND proof_as_fields -p ./target/recurse_leaf_proof -k ./target/recurse_leaf_key -o -)"
# recurse_leaf has 4 public inputs (excluding aggregation object)
PUBLIC_INPUTS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[:4]')
PROOF_AS_FIELDS=$(echo $FULL_PROOF_AS_FIELDS | jq -r '.[4:]')

RECURSE_NODE_PROVER_TOML=./recurse_node/Prover.toml
echo "key_hash = \"$VK_HASH\"" > $RECURSE_NODE_PROVER_TOML
echo "verification_key = $VK_AS_FIELDS"  >> $RECURSE_NODE_PROVER_TOML
echo "proof = $PROOF_AS_FIELDS" >> $RECURSE_NODE_PROVER_TOML
echo "public_inputs = $PUBLIC_INPUTS" >> $RECURSE_NODE_PROVER_TOML

# We can now execute and prove `recurse_node`

nargo execute recurse_node_witness --package recurse_node
$BACKEND prove -b ./target/recurse_node.json -w ./target/recurse_node_witness.gz -o ./target/recurse_node_proof

# We finally verify that the generated recursive proof is valid.
$BACKEND write_vk -b ./target/recurse_node.json -o ./target/recurse_node_key
$BACKEND verify -p ./target/recurse_node_proof -k ./target/recurse_node_key
