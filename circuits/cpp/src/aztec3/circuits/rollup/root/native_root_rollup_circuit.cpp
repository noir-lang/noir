#include "aztec3/constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/merkle_tree.hpp"
#include "init.hpp"

#include <algorithm>
#include <array>
#include <aztec3/circuits/abis/rollup/root/root_rollup_inputs.hpp>
#include <aztec3/circuits/abis/rollup/root/root_rollup_public_inputs.hpp>
#include <aztec3/circuits/abis/rollup/nullifier_leaf_preimage.hpp>
#include <aztec3/circuits/rollup/components/components.hpp>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_root_rollup {

// TODO: can we aggregate proofs if we do not have a working circuit impl
// TODO: change the public inputs array - we wont be using this?

// Access Native types through NT namespace

RootRollupPublicInputs root_rollup_circuit(DummyComposer& composer, RootRollupInputs const& rootRollupInputs)
{
    // TODO: Verify the previous rollup proofs
    // TODO: Check both previous rollup vks (in previous_rollup_data) against the permitted set of kernel vks.
    // we don't have a set of permitted kernel vks yet.

    auto left = rootRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
    auto right = rootRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

    auto aggregation_object = components::aggregate_proofs(left, right);
    components::assert_both_input_proofs_of_same_rollup_type(composer, left, right);
    components::assert_both_input_proofs_of_same_height_and_return(composer, left, right);
    components::assert_equal_constants(composer, left, right);
    components::assert_prev_rollups_follow_on_from_each_other(composer, left, right);

    // Update the historic private data tree
    auto end_tree_of_historic_private_data_tree_roots_snapshot = components::insert_subtree_to_snapshot_tree(
        composer,
        left.constants.start_tree_of_historic_private_data_tree_roots_snapshot,
        rootRollupInputs.new_historic_private_data_tree_root_sibling_path,
        fr::zero(),
        right.end_private_data_tree_snapshot.root,
        0,
        "historic private data tree roots insertion");

    // Update the historic private data tree
    auto end_tree_of_historic_contract_tree_roots_snapshot =
        components::insert_subtree_to_snapshot_tree(composer,
                                                    left.constants.start_tree_of_historic_contract_tree_roots_snapshot,
                                                    rootRollupInputs.new_historic_contract_tree_root_sibling_path,
                                                    fr::zero(),
                                                    right.end_contract_tree_snapshot.root,
                                                    0,
                                                    "historic contract tree roots insertion");

    RootRollupPublicInputs public_inputs = {
        .end_aggregation_object = aggregation_object,
        .start_private_data_tree_snapshot = left.start_private_data_tree_snapshot,
        .end_private_data_tree_snapshot = right.end_private_data_tree_snapshot,
        .start_nullifier_tree_snapshot = left.start_nullifier_tree_snapshot,
        .end_nullifier_tree_snapshot = right.end_nullifier_tree_snapshot,
        .start_contract_tree_snapshot = left.start_contract_tree_snapshot,
        .end_contract_tree_snapshot = right.end_contract_tree_snapshot,
        .start_public_data_tree_root = left.start_public_data_tree_root,
        .end_public_data_tree_root = right.end_public_data_tree_root,
        .start_tree_of_historic_private_data_tree_roots_snapshot =
            left.constants.start_tree_of_historic_private_data_tree_roots_snapshot,
        .end_tree_of_historic_private_data_tree_roots_snapshot = end_tree_of_historic_private_data_tree_roots_snapshot,
        .start_tree_of_historic_contract_tree_roots_snapshot =
            left.constants.start_tree_of_historic_contract_tree_roots_snapshot,
        .end_tree_of_historic_contract_tree_roots_snapshot = end_tree_of_historic_contract_tree_roots_snapshot,
        .calldata_hash = components::compute_calldata_hash(rootRollupInputs.previous_rollup_data),
    };

    return public_inputs;
}

} // namespace aztec3::circuits::rollup::native_root_rollup