#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "init.hpp"

#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::components {

/**
 * @brief Create an aggregation object for the proofs that are provided
 *          - We add points P0 for each of our proofs
 *          - We add points P1 for each of our proofs
 *          - We concat our public inputs
 *
 * @param mergeRollupInputs
 * @return AggregationObject
 */
AggregationObject aggregate_proofs(BaseOrMergeRollupPublicInputs const& left,
                                   BaseOrMergeRollupPublicInputs const& right)
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
void assert_both_input_proofs_of_same_rollup_type(DummyComposer& composer,
                                                  BaseOrMergeRollupPublicInputs const& left,
                                                  BaseOrMergeRollupPublicInputs const& right)
{
    composer.do_assert(left.rollup_type == right.rollup_type, "input proofs are of different rollup types");
}

/**
 * @brief Asserts that the rollup subtree heights are the same and returns the height
 *
 * @param left - The public inputs of the left rollup (base or merge)
 * @param right - The public inputs of the right rollup (base or merge)
 * @return NT::fr - The height of the rollup subtrees
 */
NT::fr assert_both_input_proofs_of_same_height_and_return(DummyComposer& composer,
                                                          BaseOrMergeRollupPublicInputs const& left,
                                                          BaseOrMergeRollupPublicInputs const& right)
{
    composer.do_assert(left.rollup_subtree_height == right.rollup_subtree_height,
                       "input proofs are of different rollup heights");
    return left.rollup_subtree_height;
}

/**
 * @brief Asserts that the constants used in the left and right child are identical
 *
 * @param left - The public inputs of the left rollup (base or merge)
 * @param right - The public inputs of the right rollup (base or merge)
 */
void assert_equal_constants(DummyComposer& composer,
                            BaseOrMergeRollupPublicInputs const& left,
                            BaseOrMergeRollupPublicInputs const& right)
{
    composer.do_assert(left.constants == right.constants, "input proofs have different constants");
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
void assert_prev_rollups_follow_on_from_each_other(DummyComposer& composer,
                                                   BaseOrMergeRollupPublicInputs const& left,
                                                   BaseOrMergeRollupPublicInputs const& right)
{
    composer.do_assert(left.end_private_data_tree_snapshot == right.start_private_data_tree_snapshot,
                       "input proofs have different private data tree snapshots");
    composer.do_assert(left.end_nullifier_tree_snapshot == right.start_nullifier_tree_snapshot,
                       "input proofs have different nullifier tree snapshots");
    composer.do_assert(left.end_contract_tree_snapshot == right.start_contract_tree_snapshot,
                       "input proofs have different contract tree snapshots");
}

} // namespace aztec3::circuits::rollup::components