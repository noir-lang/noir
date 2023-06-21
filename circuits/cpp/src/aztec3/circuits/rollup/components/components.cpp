#include "init.hpp"

#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include <barretenberg/barretenberg.hpp>

#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::components {

/**
 * @brief Get the root of an empty tree of a given depth
 *
 * @param depth
 * @return NT::fr
 */
NT::fr calculate_empty_tree_root(const size_t depth)
{
    stdlib::merkle_tree::MemoryTree const empty_tree = stdlib::merkle_tree::MemoryTree(depth);
    return empty_tree.root();
}

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
    composer.do_assert(left.rollup_type == right.rollup_type,
                       "input proofs are of different rollup types",
                       utils::CircuitErrorCode::ROLLUP_TYPE_MISMATCH);
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
                       "input proofs are of different rollup heights",
                       utils::CircuitErrorCode::ROLLUP_HEIGHT_MISMATCH);
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
    composer.do_assert(left.constants == right.constants,
                       "input proofs have different constants",
                       utils::CircuitErrorCode::CONSTANTS_MISMATCH);
}

/**
 * @brief Computes the calldata hash for a base rollup
 *
 * @param kernel_data - 2 kernels
 * @return calldata hash stored in 2 fields
 */
std::array<fr, NUM_FIELDS_PER_SHA256> compute_kernels_calldata_hash(
    std::array<abis::PreviousKernelData<NT>, 2> kernel_data)
{
    // Compute calldata hashes
    // Consist of 2 kernels
    // 8 commitments (4 per kernel) -> 8 fields
    // 8 nullifiers (4 per kernel) -> 8 fields
    // 8 public data update requests (4 per kernel) -> 16 fields
    // 4 l2 -> l1 messages (2 per kernel) -> 4 fields
    // 2 contract deployments (1 per kernel) -> 6 fields
    // 2 encrypted logs hashes (1 per kernel) -> 4 fields --> 2 sha256 hashes --> 64 bytes
    // 2 unencrypted logs hashes (1 per kernel) -> 4 fields --> 2 sha256 hashes --> 64 bytes
    auto const number_of_inputs =
        (KERNEL_NEW_COMMITMENTS_LENGTH + KERNEL_NEW_NULLIFIERS_LENGTH + KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * 2 +
         KERNEL_NEW_L2_TO_L1_MSGS_LENGTH + KERNEL_NEW_CONTRACTS_LENGTH * 3 + KERNEL_NUM_ENCRYPTED_LOGS_HASHES * 2 +
         KERNEL_NUM_UNENCRYPTED_LOGS_HASHES * 2) *
        2;
    std::array<NT::fr, number_of_inputs> calldata_hash_inputs;

    for (size_t i = 0; i < 2; i++) {
        auto new_commitments = kernel_data[i].public_inputs.end.new_commitments;
        auto new_nullifiers = kernel_data[i].public_inputs.end.new_nullifiers;
        auto public_data_update_requests = kernel_data[i].public_inputs.end.public_data_update_requests;
        auto newL2ToL1msgs = kernel_data[i].public_inputs.end.new_l2_to_l1_msgs;
        auto encryptedLogsHash = kernel_data[i].public_inputs.end.encrypted_logs_hash;
        auto unencryptedLogsHash = kernel_data[i].public_inputs.end.unencrypted_logs_hash;

        size_t offset = 0;

        for (size_t j = 0; j < KERNEL_NEW_COMMITMENTS_LENGTH; j++) {
            calldata_hash_inputs[offset + i * KERNEL_NEW_COMMITMENTS_LENGTH + j] = new_commitments[j];
        }
        offset += KERNEL_NEW_COMMITMENTS_LENGTH * 2;

        for (size_t j = 0; j < KERNEL_NEW_NULLIFIERS_LENGTH; j++) {
            calldata_hash_inputs[offset + i * KERNEL_NEW_NULLIFIERS_LENGTH + j] = new_nullifiers[j];
        }
        offset += KERNEL_NEW_NULLIFIERS_LENGTH * 2;

        for (size_t j = 0; j < KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH; j++) {
            calldata_hash_inputs[offset + i * KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * 2 + j * 2] =
                public_data_update_requests[j].leaf_index;
            calldata_hash_inputs[offset + i * KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * 2 + j * 2 + 1] =
                public_data_update_requests[j].new_value;
        }
        offset += KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * 2 * 2;

        for (size_t j = 0; j < KERNEL_NEW_L2_TO_L1_MSGS_LENGTH; j++) {
            calldata_hash_inputs[offset + i * KERNEL_NEW_L2_TO_L1_MSGS_LENGTH + j] = newL2ToL1msgs[j];
        }
        offset += KERNEL_NEW_L2_TO_L1_MSGS_LENGTH * 2;

        auto const contract_leaf = kernel_data[i].public_inputs.end.new_contracts[0];
        calldata_hash_inputs[offset + i] = contract_leaf.hash();

        offset += KERNEL_NEW_CONTRACTS_LENGTH * 2;

        auto new_contracts = kernel_data[i].public_inputs.end.new_contracts;
        calldata_hash_inputs[offset + i * 2] = new_contracts[0].contract_address;
        calldata_hash_inputs[offset + i * 2 + 1] = new_contracts[0].portal_contract_address;

        offset += KERNEL_NEW_CONTRACTS_LENGTH * 2 * 2;

        calldata_hash_inputs[offset + i * 2] = encryptedLogsHash[0];
        calldata_hash_inputs[offset + i * 2 + 1] = encryptedLogsHash[1];

        offset += KERNEL_NUM_ENCRYPTED_LOGS_HASHES * 2 * 2;

        calldata_hash_inputs[offset + i * 2] = unencryptedLogsHash[0];
        calldata_hash_inputs[offset + i * 2 + 1] = unencryptedLogsHash[1];
    }

    // We subtract 4 from inputs size because 1 logs hash is stored in 2 fields and those 2 fields get converted only
    // to 256 bits and there are 4 logs hashes in total.
    constexpr auto num_bytes = (calldata_hash_inputs.size() - 4) * 32;
    std::array<uint8_t, num_bytes> calldata_hash_inputs_bytes;
    // Convert all into a buffer, then copy into the array, then hash
    for (size_t i = 0; i < calldata_hash_inputs.size() - 4; i++) {  // -4 because logs are processed out of the loop
        auto as_bytes = calldata_hash_inputs[i].to_buffer();

        auto offset = i * 32;
        std::copy(as_bytes.begin(), as_bytes.end(), calldata_hash_inputs_bytes.begin() + offset);
    }

    // Copy the 4 fields of 2 encrypted logs to 64 bytes
    // Modified version of:
    // https://github.com/AztecProtocol/aztec-packages/blob/01080c7f1d2956512b6a9cff0582b43be25b3cc2/circuits/cpp/src/aztec3/circuits/hash.hpp#L350
    const uint32_t encrypted_logs_start_index = calldata_hash_inputs.size() - 8;
    const uint32_t first_modified_byte_encrypted = num_bytes - 128;  // 128 = num bytes occupied by all the logs hashes
    for (uint8_t i = 0; i < 4; i++) {
        auto half = calldata_hash_inputs[encrypted_logs_start_index + i].to_buffer();
        for (uint8_t j = 0; j < 16; j++) {
            calldata_hash_inputs_bytes[first_modified_byte_encrypted + i * 16 + j] = half[16 + j];
        }
    }

    // Do the same for the unencrypted logs
    const uint32_t unencrypted_logs_start_index = calldata_hash_inputs.size() - 4;
    const uint32_t first_modified_byte_unencrypted =
        num_bytes - 64;  // 64 = num bytes occupied by unencrypted logs hashes
    for (uint8_t i = 0; i < 4; i++) {
        auto half = calldata_hash_inputs[unencrypted_logs_start_index + i].to_buffer();
        for (uint8_t j = 0; j < 16; j++) {
            calldata_hash_inputs_bytes[first_modified_byte_unencrypted + i * 16 + j] = half[16 + j];
        }
    }

    std::vector<uint8_t> const calldata_hash_inputs_bytes_vec(calldata_hash_inputs_bytes.begin(),
                                                              calldata_hash_inputs_bytes.end());

    auto h = sha256::sha256(calldata_hash_inputs_bytes_vec);

    // Split the hash into two fields, a high and a low
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

    return std::array<NT::fr, NUM_FIELDS_PER_SHA256>{ high, low };
}

/**
 * @brief From two previous rollup data, compute a single calldata hash
 *
 * @param previous_rollup_data
 * @return calldata hash stored in 2 fields
 */
std::array<fr, NUM_FIELDS_PER_SHA256> compute_calldata_hash(
    std::array<abis::PreviousRollupData<NT>, 2> previous_rollup_data)
{
    return accumulate_sha256<NT>({ previous_rollup_data[0].base_or_merge_rollup_public_inputs.calldata_hash[0],
                                   previous_rollup_data[0].base_or_merge_rollup_public_inputs.calldata_hash[1],
                                   previous_rollup_data[1].base_or_merge_rollup_public_inputs.calldata_hash[0],
                                   previous_rollup_data[1].base_or_merge_rollup_public_inputs.calldata_hash[1] });
}

// asserts that the end snapshot of previous_rollup 0 equals the start snapshot of previous_rollup 1 (i.e. ensure they
// follow on from one-another). Ensures that right uses the tres that was updated by left.
void assert_prev_rollups_follow_on_from_each_other(DummyComposer& composer,
                                                   BaseOrMergeRollupPublicInputs const& left,
                                                   BaseOrMergeRollupPublicInputs const& right)
{
    composer.do_assert(left.end_private_data_tree_snapshot == right.start_private_data_tree_snapshot,
                       "input proofs have different private data tree snapshots",
                       utils::CircuitErrorCode::PRIVATE_DATA_TREE_SNAPSHOT_MISMATCH);
    composer.do_assert(left.end_nullifier_tree_snapshot == right.start_nullifier_tree_snapshot,
                       "input proofs have different nullifier tree snapshots",
                       utils::CircuitErrorCode::NULLIFIER_TREE_SNAPSHOT_MISMATCH);
    composer.do_assert(left.end_contract_tree_snapshot == right.start_contract_tree_snapshot,
                       "input proofs have different contract tree snapshots",
                       utils::CircuitErrorCode::CONTRACT_TREE_SNAPSHOT_MISMATCH);
    composer.do_assert(left.end_public_data_tree_root == right.start_public_data_tree_root,
                       "input proofs have different public data tree snapshots",
                       utils::CircuitErrorCode::CONTRACT_TREE_SNAPSHOT_MISMATCH);
}

}  // namespace aztec3::circuits::rollup::components