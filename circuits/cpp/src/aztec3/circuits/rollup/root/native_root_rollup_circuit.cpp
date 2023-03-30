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
#include <cassert>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_root_rollup {

// TODO: can we aggregate proofs if we do not have a working circuit impl
// TODO: change the public inputs array - we wont be using this?

// Access Native types through NT namespace

bool verify_merge_proof(NT::Proof merge_proof)
{
    (void)merge_proof;
    return true;
}

AggregationObject aggregate_proofs(RootRollupInputs const& rootRollupInputs)
{
    // TODO: NOTE: for now we simply return the aggregation object from the first proof
    return rootRollupInputs.previous_rollup_data[0].base_rollup_public_inputs.end_aggregation_object;
}

bool is_constants_equal(ConstantRollupData left, ConstantRollupData right)
{
    return left == right;
}

template <size_t N>
NT::fr iterate_through_tree_via_sibling_path(NT::fr leaf,
                                             NT::uint32 leafIndex,
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
void check_membership(NT::fr leaf, NT::uint32 leafIndex, std::array<NT::fr, N> const& siblingPath, NT::fr root)
{
    auto calculatedRoot = iterate_through_tree_via_sibling_path(leaf, leafIndex, siblingPath);
    if (calculatedRoot != root) {
        // throw std::runtime_error("Merkle membership check failed");
    }
}

std::array<fr, 2> compute_calldata_hash(RootRollupInputs const& rootRollupInputs)
{

    // Compute the calldata hash
    std::array<uint8_t, 2 * 32> calldata_hash_input_bytes;
    for (uint8_t i = 0; i < 2; i++) {
        std::array<fr, 2> calldata_hash_fr =
            rootRollupInputs.previous_rollup_data[i].base_rollup_public_inputs.calldata_hash;

        auto high_buffer = calldata_hash_fr[0].to_buffer();
        auto low_buffer = calldata_hash_fr[1].to_buffer();

        for (uint8_t j = 0; j < 16; ++j) {
            calldata_hash_input_bytes[i * 32 + j] = high_buffer[16 + j];
            calldata_hash_input_bytes[i * 32 + 16 + j] = low_buffer[16 + j];
        }
    }

    std::vector<uint8_t> calldata_hash_input_bytes_vec(calldata_hash_input_bytes.begin(),
                                                       calldata_hash_input_bytes.end());

    auto h = sha256::sha256(calldata_hash_input_bytes_vec);

    // Split the hash into two fields, a high and a low
    std::array<uint8_t, 32> buf_1, buf_2;
    for (uint8_t i = 0; i < 16; i++) {
        buf_1[i] = 0;
        buf_1[16 + i] = h[i];
        buf_2[i] = 0;
        buf_2[16 + i] = h[i + 16];
    }
    auto high = fr::serialize_from_buffer(buf_1.data());
    auto low = fr::serialize_from_buffer(buf_2.data());

    return { high, low };
}

template <size_t N>
AppendOnlySnapshot insert_at_empty_in_snapshot_tree(AppendOnlySnapshot const& old_snapshot,
                                                    std::array<NT::fr, N> const& siblingPath,
                                                    NT::fr subtreeRootToInsert)
{
    // check that the value is zero at the path (unused)
    check_membership(fr::zero(), old_snapshot.next_available_leaf_index, siblingPath, old_snapshot.root);

    // Compute the new root after the update
    auto new_root =
        iterate_through_tree_via_sibling_path(subtreeRootToInsert, old_snapshot.next_available_leaf_index, siblingPath);

    return { .root = new_root, .next_available_leaf_index = old_snapshot.next_available_leaf_index + 1 };
}

// Important types:
//   - BaseRollupPublicInputs - where we want to put our return values
//
// TODO: replace auto
RootRollupPublicInputs root_rollup_circuit(RootRollupInputs const& rootRollupInputs)
{
    // TODO: Check the historic trees as well
    // old -> leftmost
    // new -> rightmost

    AggregationObject aggregation_object = aggregate_proofs(rootRollupInputs);

    // Verify the previous merge proofs (for now these are actually base proofs)
    for (size_t i = 0; i < 2; i++) {
        NT::Proof proof = rootRollupInputs.previous_rollup_data[i].proof;
        assert(verify_merge_proof(proof));
    }

    auto left = rootRollupInputs.previous_rollup_data[0].base_rollup_public_inputs;
    auto right = rootRollupInputs.previous_rollup_data[1].base_rollup_public_inputs;

    // Constants must be the same between left and right
    assert(is_constants_equal(left.constants, right.constants));

    // Update the historic private data tree
    AppendOnlySnapshot end_tree_of_historic_private_data_tree_roots_snapshot =
        insert_at_empty_in_snapshot_tree(left.constants.start_tree_of_historic_private_data_tree_roots_snapshot,
                                         rootRollupInputs.new_historic_private_data_tree_root_sibling_path,
                                         right.end_private_data_tree_snapshot.root);

    // Update the historic private data tree
    AppendOnlySnapshot end_tree_of_historic_contract_tree_roots_snapshot =
        insert_at_empty_in_snapshot_tree(left.constants.start_tree_of_historic_contract_tree_roots_snapshot,
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
        .calldata_hash = compute_calldata_hash(rootRollupInputs),
    };

    return public_inputs;
}

} // namespace aztec3::circuits::rollup::native_root_rollup