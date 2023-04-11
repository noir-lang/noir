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
#include <aztec3/circuits/rollup/merge/native_merge_rollup_circuit.hpp>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_root_rollup {

// TODO: can we aggregate proofs if we do not have a working circuit impl
// TODO: change the public inputs array - we wont be using this?

// Access Native types through NT namespace

template <size_t N>
NT::fr iterate_through_tree_via_sibling_path(NT::fr leaf,
                                             NT::uint32 const& leafIndex,
                                             std::array<NT::fr, N> const& siblingPath)
{
    for (size_t i = 0; i < siblingPath.size(); i++) {
        if (leafIndex & (1 << i)) {
            leaf = crypto::pedersen_hash::hash_multiple({ siblingPath[i], leaf });
        } else {
            leaf = crypto::pedersen_hash::hash_multiple({ leaf, siblingPath[i] });
        }
    }
    return leaf;
}

template <size_t N>
void check_membership(DummyComposer& composer,
                      NT::fr const& leaf,
                      NT::uint32 const& leafIndex,
                      std::array<NT::fr, N> const& siblingPath,
                      NT::fr const& root)
{
    auto computed_root = iterate_through_tree_via_sibling_path(leaf, leafIndex, siblingPath);
    composer.do_assert(root == computed_root, "Membership check failed");
}

template <size_t N>
AppendOnlySnapshot insert_at_empty_in_snapshot_tree(DummyComposer& composer,
                                                    AppendOnlySnapshot const& old_snapshot,
                                                    std::array<NT::fr, N> const& siblingPath,
                                                    NT::fr leaf)
{
    // check that the value is zero at the path (unused)
    // TODO: We should be able to actually skip this, because the contract will be indirectly enforce it through
    // old_snapshot.next_available_leaf_index
    check_membership(composer, fr::zero(), old_snapshot.next_available_leaf_index, siblingPath, old_snapshot.root);

    // Compute the new root after the update
    auto new_root = iterate_through_tree_via_sibling_path(leaf, old_snapshot.next_available_leaf_index, siblingPath);

    return { .root = new_root, .next_available_leaf_index = old_snapshot.next_available_leaf_index + 1 };
}

// Important types:
//   - BaseRollupPublicInputs - where we want to put our return values
//
// TODO: replace auto
RootRollupPublicInputs root_rollup_circuit(aztec3::utils::DummyComposer& composer,
                                           RootRollupInputs const& rootRollupInputs)
{
    // TODO: Verify the previous rollup proofs
    // TODO: Check both previous rollup vks (in previous_rollup_data) against the permitted set of kernel vks.
    // we don't have a set of permitted kernel vks yet.

    auto left = rootRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
    auto right = rootRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

    AggregationObject aggregation_object = native_merge_rollup::aggregate_proofs(left, right);
    native_merge_rollup::assert_both_input_proofs_of_same_rollup_type(composer, left, right);
    native_merge_rollup::assert_both_input_proofs_of_same_height_and_return(composer, left, right);
    native_merge_rollup::assert_equal_constants(composer, left, right);
    native_merge_rollup::assert_prev_rollups_follow_on_from_each_other(composer, left, right);

    // Update the historic private data tree
    AppendOnlySnapshot end_tree_of_historic_private_data_tree_roots_snapshot =
        insert_at_empty_in_snapshot_tree(composer,
                                         left.constants.start_tree_of_historic_private_data_tree_roots_snapshot,
                                         rootRollupInputs.new_historic_private_data_tree_root_sibling_path,
                                         right.end_private_data_tree_snapshot.root);

    // Update the historic private data tree
    AppendOnlySnapshot end_tree_of_historic_contract_tree_roots_snapshot =
        insert_at_empty_in_snapshot_tree(composer,
                                         left.constants.start_tree_of_historic_contract_tree_roots_snapshot,
                                         rootRollupInputs.new_historic_contract_tree_root_sibling_path,
                                         right.end_contract_tree_snapshot.root);

    RootRollupPublicInputs public_inputs = {
        .end_aggregation_object = aggregation_object,
        .start_private_data_tree_snapshot = left.start_private_data_tree_snapshot,
        .end_private_data_tree_snapshot = right.end_private_data_tree_snapshot,
        .start_nullifier_tree_snapshot = left.start_nullifier_tree_snapshot,
        .end_nullifier_tree_snapshot = right.end_nullifier_tree_snapshot,
        .start_contract_tree_snapshot = left.start_contract_tree_snapshot,
        .end_contract_tree_snapshot = right.end_contract_tree_snapshot,
        .start_tree_of_historic_private_data_tree_roots_snapshot =
            left.constants.start_tree_of_historic_private_data_tree_roots_snapshot,
        .end_tree_of_historic_private_data_tree_roots_snapshot = end_tree_of_historic_private_data_tree_roots_snapshot,
        .start_tree_of_historic_contract_tree_roots_snapshot =
            left.constants.start_tree_of_historic_contract_tree_roots_snapshot,
        .end_tree_of_historic_contract_tree_roots_snapshot = end_tree_of_historic_contract_tree_roots_snapshot,
        .calldata_hash = native_merge_rollup::compute_calldata_hash(rootRollupInputs.previous_rollup_data),
    };

    return public_inputs;
}

} // namespace aztec3::circuits::rollup::native_root_rollup