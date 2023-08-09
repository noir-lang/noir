#include "init.hpp"

#include "aztec3/circuits/abis/rollup/root/root_rollup_inputs.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_public_inputs.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/rollup/components/components.hpp"
#include "aztec3/constants.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_root_rollup {

// Used when calling library functions like `check_membership` which have their own generic error code.
// So we pad this in front of the error message to identify where the error originally came from.
const std::string ROOT_CIRCUIT_ERROR_MESSAGE_BEGINNING = "root_rollup_circuit: ";

// TODO: can we aggregate proofs if we do not have a working circuit impl
// TODO: change the public inputs array - we wont be using this?

// Access Native types through NT namespace

/**
 * @brief Calculates the messages subtree from the leaves array
 * @param leaves
 * @return root
 */
NT::fr calculate_subtree(std::array<NT::fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> leaves)
{
    MemoryStore merkle_tree_store;
    MerkleTree merkle_tree(merkle_tree_store, L1_TO_L2_MSG_SUBTREE_HEIGHT);

    for (size_t i = 0; i < NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP; i++) {
        merkle_tree.update_element(i, leaves[i]);
    }
    return merkle_tree.root();
}

/**
 * @brief Computes the messages hash from the leaves array
 * @param leaves
 * @param return - hash split into two field elements
 */
std::array<NT::fr, NUM_FIELDS_PER_SHA256> compute_messages_hash(
    std::array<NT::fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> leaves)
{
    // convert vector of field elements into uint_8
    std::array<uint8_t, 32 * NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> messages_hash_input_bytes;
    for (size_t i = 0; i < NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP; i++) {
        auto bytes = leaves[i].to_buffer();
        for (size_t j = 0; j < 32; j++) {
            messages_hash_input_bytes[i * 32 + j] = bytes[j];
        }
    }

    std::vector<uint8_t> const messages_hash_input_bytes_vec(messages_hash_input_bytes.begin(),
                                                             messages_hash_input_bytes.end());
    auto h = sha256::sha256(messages_hash_input_bytes_vec);

    std::array<uint8_t, 32> buf_1;
    std::array<uint8_t, 32> buf_2;
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

RootRollupPublicInputs root_rollup_circuit(DummyBuilder& builder, RootRollupInputs const& rootRollupInputs)
{
    // TODO: Verify the previous rollup proofs
    // TODO: Check both previous rollup vks (in previous_rollup_data) against the permitted set of kernel vks.
    // we don't have a set of permitted kernel vks yet.

    auto left = rootRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
    auto right = rootRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

    auto aggregation_object = components::aggregate_proofs(left, right);
    components::assert_both_input_proofs_of_same_rollup_type(builder, left, right);
    components::assert_both_input_proofs_of_same_height_and_return(builder, left, right);
    components::assert_equal_constants(builder, left, right);
    components::assert_prev_rollups_follow_on_from_each_other(builder, left, right);

    // Check correct l1 to l2 tree given
    // Compute subtree inserting l1 to l2 messages
    auto l1_to_l2_subtree_root = calculate_subtree(rootRollupInputs.l1_to_l2_messages);

    // Insert subtree into the l1 to l2 data tree
    const auto empty_l1_to_l2_subtree_root = components::calculate_empty_tree_root(L1_TO_L2_MSG_SUBTREE_HEIGHT);
    auto new_l1_to_l2_messages_tree_snapshot = components::insert_subtree_to_snapshot_tree(
        builder,
        rootRollupInputs.start_l1_to_l2_message_tree_snapshot,
        rootRollupInputs.new_l1_to_l2_message_tree_root_sibling_path,
        empty_l1_to_l2_subtree_root,
        l1_to_l2_subtree_root,
        L1_TO_L2_MSG_SUBTREE_HEIGHT,
        format(ROOT_CIRCUIT_ERROR_MESSAGE_BEGINNING,
               "l1 to l2 message tree not empty at location where subtree would be inserted"));

    // Build the block hash for this iteration from the tree roots and global variables
    // Then insert the block into the historic blocks tree
    auto block_hash = compute_block_hash_with_globals(left.constants.global_variables,
                                                      right.end_private_data_tree_snapshot.root,
                                                      right.end_nullifier_tree_snapshot.root,
                                                      right.end_contract_tree_snapshot.root,
                                                      new_l1_to_l2_messages_tree_snapshot.root,
                                                      right.end_public_data_tree_root);

    // Update the historic blocks tree
    auto end_historic_blocks_tree_snapshot = components::insert_subtree_to_snapshot_tree(
        builder,
        rootRollupInputs.start_historic_blocks_tree_snapshot,
        rootRollupInputs.new_historic_blocks_tree_sibling_path,
        fr::zero(),
        block_hash,
        0,
        format(ROOT_CIRCUIT_ERROR_MESSAGE_BEGINNING,
               "historic blocks tree roots not empty at location where subtree would be inserted"));


    RootRollupPublicInputs public_inputs = {
        .end_aggregation_object = aggregation_object,
        .globalVariables = left.constants.global_variables,
        .start_private_data_tree_snapshot = left.start_private_data_tree_snapshot,
        .end_private_data_tree_snapshot = right.end_private_data_tree_snapshot,
        .start_nullifier_tree_snapshot = left.start_nullifier_tree_snapshot,
        .end_nullifier_tree_snapshot = right.end_nullifier_tree_snapshot,
        .start_contract_tree_snapshot = left.start_contract_tree_snapshot,
        .end_contract_tree_snapshot = right.end_contract_tree_snapshot,
        .start_public_data_tree_root = left.start_public_data_tree_root,
        .end_public_data_tree_root = right.end_public_data_tree_root,
        .start_l1_to_l2_messages_tree_snapshot = rootRollupInputs.start_l1_to_l2_message_tree_snapshot,
        .end_l1_to_l2_messages_tree_snapshot = new_l1_to_l2_messages_tree_snapshot,
        .start_historic_blocks_tree_snapshot = rootRollupInputs.start_historic_blocks_tree_snapshot,
        .end_historic_blocks_tree_snapshot = end_historic_blocks_tree_snapshot,
        .calldata_hash = components::compute_calldata_hash(rootRollupInputs.previous_rollup_data),
        .l1_to_l2_messages_hash = compute_messages_hash(rootRollupInputs.l1_to_l2_messages)
    };

    return public_inputs;
}

}  // namespace aztec3::circuits::rollup::native_root_rollup