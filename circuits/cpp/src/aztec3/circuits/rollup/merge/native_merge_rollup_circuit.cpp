#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
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
#include <cassert>
#include <cstdint>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_merge_rollup {

/**
 * @brief Create an aggregation object for the proofs that are provided
 *          - We add points P0 for each of our proofs
 *          - We add points P1 for each of our proofs
 *          - We concat our public inputs
 *
 * @param mergeRollupInputs
 * @return AggregationObject
 */
AggregationObject aggregate_proofs(BaseOrMergeRollupPublicInputs left, BaseOrMergeRollupPublicInputs right)
{
    // TODO: NOTE: for now we simply return the aggregation object from the first proof
    (void)right;
    return left.end_aggregation_object;
}

/**
 * @brief Asserts that the rollup types are the same
 *
 * @param left - The public inputs of the left rollup (base or merge)
 * @param right - The public inputs of the right rollup (base or merge)
 */
void assert_both_input_proofs_of_same_rollup_type(BaseOrMergeRollupPublicInputs left,
                                                  BaseOrMergeRollupPublicInputs right)
{
    assert(left.rollup_type == right.rollup_type);
    (void)left;
    (void)right;
}

/**
 * @brief Asserts that the rollup subtree heights are the same and returns the height
 *
 * @param left - The public inputs of the left rollup (base or merge)
 * @param right - The public inputs of the right rollup (base or merge)
 * @return NT::fr - The height of the rollup subtrees
 */
NT::fr assert_both_input_proofs_of_same_height_and_return(BaseOrMergeRollupPublicInputs left,
                                                          BaseOrMergeRollupPublicInputs right)
{
    assert(left.rollup_subtree_height == right.rollup_subtree_height);
    (void)left;
    (void)right;
    return left.rollup_subtree_height;
}

/**
 * @brief Asserts that the constants used in the left and right child are identical
 *
 * @param left - The public inputs of the left rollup (base or merge)
 * @param right - The public inputs of the right rollup (base or merge)
 */
void assert_equal_constants(BaseOrMergeRollupPublicInputs left, BaseOrMergeRollupPublicInputs right)
{
    assert(left.constants == right.constants);
    (void)left;
    (void)right;
}

// Generates a 512 bit input from right and left 256 bit hashes. Then computes the sha256, and splits the hash into two
// field elements, a high and a low that is returned.
std::array<fr, 2> compute_calldata_hash(std::array<abis::PreviousRollupData<NT>, 2> previous_rollup_data)
{
    // Generate a 512 bit input from right and left 256 bit hashes
    std::array<uint8_t, 2 * 32> calldata_hash_input_bytes;
    for (uint8_t i = 0; i < 2; i++) {
        std::array<fr, 2> calldata_hash_fr = previous_rollup_data[i].base_or_merge_rollup_public_inputs.calldata_hash;

        auto high_buffer = calldata_hash_fr[0].to_buffer();
        auto low_buffer = calldata_hash_fr[1].to_buffer();

        for (uint8_t j = 0; j < 16; ++j) {
            calldata_hash_input_bytes[i * 32 + j] = high_buffer[16 + j];
            calldata_hash_input_bytes[i * 32 + 16 + j] = low_buffer[16 + j];
        }
    }

    // Compute the sha256
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

// asserts that the end snapshot of previous_rollup 0 equals the start snapshot of previous_rollup 1 (i.e. ensure they
// follow on from one-another). Ensures that right uses the tres that was updated by left.
void assert_prev_rollups_follow_on_from_each_other(BaseOrMergeRollupPublicInputs left,
                                                   BaseOrMergeRollupPublicInputs right)
{
    assert(left.end_private_data_tree_snapshot == right.start_private_data_tree_snapshot);
    assert(left.end_nullifier_tree_snapshot == right.start_nullifier_tree_snapshot);
    assert(left.end_contract_tree_snapshot == right.start_contract_tree_snapshot);
    // void variables since despite using in assert, it says, "unused variable"
    (void)left;
    (void)right;
}

BaseOrMergeRollupPublicInputs merge_rollup_circuit(MergeRollupInputs mergeRollupInputs)
{
    // TODO: Verify the previous rollup proofs
    // TODO: Check both previous rollup vks (in previous_rollup_data) against the permitted set of kernel vks.
    // we don't have a set of permitted kernel vks yet.

    auto left = mergeRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
    auto right = mergeRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

    // check that both input proofs are either both "BASE" or "MERGE" and not a mix!
    // this prevents having wonky commitment, nullifier and contract subtrees.
    AggregationObject aggregation_object = aggregate_proofs(left, right);
    assert_both_input_proofs_of_same_rollup_type(left, right);
    auto current_height = assert_both_input_proofs_of_same_height_and_return(left, right);
    assert_equal_constants(left, right);
    assert_prev_rollups_follow_on_from_each_other(left, right);

    // compute calldata hash:
    auto new_calldata_hash = compute_calldata_hash(mergeRollupInputs.previous_rollup_data);

    BaseOrMergeRollupPublicInputs public_inputs = {
        .rollup_type = abis::MERGE_ROLLUP_TYPE,
        .rollup_subtree_height = current_height + 1,
        .end_aggregation_object = aggregation_object,
        .constants = left.constants,
        .start_private_data_tree_snapshot = left.start_private_data_tree_snapshot,
        .end_private_data_tree_snapshot = right.end_private_data_tree_snapshot,
        .start_nullifier_tree_snapshot = left.start_nullifier_tree_snapshot,
        .end_nullifier_tree_snapshot = right.end_nullifier_tree_snapshot,
        .start_contract_tree_snapshot = left.start_contract_tree_snapshot,
        .end_contract_tree_snapshot = right.end_contract_tree_snapshot,
        .calldata_hash = new_calldata_hash,
    };

    return public_inputs;
}

} // namespace aztec3::circuits::rollup::native_merge_rollup