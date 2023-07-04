#pragma once
#include <stddef.h>

namespace aztec3 {

/**
 * @brief Computes log2 at compile-time for inputs of the form 2^n.
 *
 * @param input
 * @return ⌈ log₂(input) ⌉
 */
constexpr size_t log2(size_t input)
{
    return (input < 2) ? 0 : 1 + log2(input / 2);
}


// Note: must be kept in sync with ts/structs/constants.ts
constexpr size_t ARGS_LENGTH = 16;
constexpr size_t RETURN_VALUES_LENGTH = 4;

constexpr size_t READ_REQUESTS_LENGTH = 4;


/**
 * Note: The number of commitments that 1 function call can output is: NEW_COMMITMENTS_LENGTH = 4. The number of
 * nullifiers that 1 function call can output is: NEW_NULLIFIERS_LENGTH = 4. This is different from
 * KERNEL_NEW_COMMITMENTS_LENGTH and KERNEL_NEW_NULLIFIERS_LENGTH because, in the kernel circuits, we accumulate the
 * commitments and the nullifiers from all functions calls in a transaction. Therefore, we always must have:
 *
 * KERNEL_NEW_COMMITMENTS_LENGTH ≥ NEW_COMMITMENTS_LENGTH
 * KERNEL_NEW_NULLIFIERS_LENGTH ≥ NEW_NULLIFIERS_LENGTH
 *
 */
// TODO(962): Rename this to `COMMITMENTS_PER_KERNEL` and make it consistent across the codebase.
constexpr size_t NEW_COMMITMENTS_LENGTH = 4;
constexpr size_t NEW_NULLIFIERS_LENGTH = 4;

constexpr size_t PRIVATE_CALL_STACK_LENGTH = 4;
constexpr size_t PUBLIC_CALL_STACK_LENGTH = 4;
constexpr size_t NEW_L2_TO_L1_MSGS_LENGTH = 2;

constexpr size_t KERNEL_NEW_COMMITMENTS_LENGTH = PRIVATE_CALL_STACK_LENGTH * NEW_COMMITMENTS_LENGTH;
constexpr size_t KERNEL_NEW_NULLIFIERS_LENGTH = 4;
constexpr size_t KERNEL_NEW_CONTRACTS_LENGTH = 1;
constexpr size_t KERNEL_PRIVATE_CALL_STACK_LENGTH = 8;
constexpr size_t KERNEL_PUBLIC_CALL_STACK_LENGTH = 8;
constexpr size_t KERNEL_NEW_L2_TO_L1_MSGS_LENGTH = 2;
constexpr size_t KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH = 4;
constexpr size_t KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH = 4;
constexpr size_t KERNEL_PUBLIC_DATA_READS_LENGTH = 4;
constexpr size_t KERNEL_NUM_ENCRYPTED_LOGS_HASHES = 1;
constexpr size_t KERNEL_NUM_UNENCRYPTED_LOGS_HASHES = 1;

constexpr size_t VK_TREE_HEIGHT = 3;
constexpr size_t FUNCTION_TREE_HEIGHT = 4;
constexpr size_t CONTRACT_TREE_HEIGHT = 8;
constexpr size_t PRIVATE_DATA_TREE_HEIGHT = 16;
constexpr size_t NULLIFIER_TREE_HEIGHT = 8;
constexpr size_t PUBLIC_DATA_TREE_HEIGHT = 254;
constexpr size_t L1_TO_L2_MSG_TREE_HEIGHT = 8;

constexpr size_t CONTRACT_SUBTREE_DEPTH = 1;
constexpr size_t CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH = CONTRACT_TREE_HEIGHT - CONTRACT_SUBTREE_DEPTH;

// TODO(961): Use this constant everywhere instead of hard-coded "2".
constexpr size_t KERNELS_PER_ROLLUP = 2;
constexpr size_t PRIVATE_DATA_SUBTREE_DEPTH =
    static_cast<size_t>(log2(KERNELS_PER_ROLLUP * KERNEL_NEW_COMMITMENTS_LENGTH));
constexpr size_t PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH = PRIVATE_DATA_TREE_HEIGHT - PRIVATE_DATA_SUBTREE_DEPTH;

constexpr size_t NULLIFIER_SUBTREE_DEPTH = static_cast<size_t>(log2(KERNELS_PER_ROLLUP * KERNEL_NEW_NULLIFIERS_LENGTH));
constexpr size_t NULLIFIER_SUBTREE_INCLUSION_CHECK_DEPTH = NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_DEPTH;

// NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP must equal 2^L1_TO_L2_MSG_SUBTREE_DEPTH for subtree insertions.
constexpr size_t L1_TO_L2_MSG_SUBTREE_DEPTH = 4;
constexpr size_t NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP = 16;
constexpr size_t L1_TO_L2_MSG_SUBTREE_INCLUSION_CHECK_DEPTH = L1_TO_L2_MSG_TREE_HEIGHT - L1_TO_L2_MSG_SUBTREE_DEPTH;

constexpr size_t PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT = 8;
constexpr size_t CONTRACT_TREE_ROOTS_TREE_HEIGHT = 8;
constexpr size_t L1_TO_L2_MSG_TREE_ROOTS_TREE_HEIGHT = 8;
constexpr size_t ROLLUP_VK_TREE_HEIGHT = 8;  // TODO: update

constexpr size_t FUNCTION_SELECTOR_NUM_BYTES = 4;  // must be <= 31

// sha256 hash is stored in two fields to accommodate all 256-bits of the hash
constexpr size_t NUM_FIELDS_PER_SHA256 = 2;

/**
 * Enumerate the hash_indices which are used for pedersen hashing.
 * We start from 1 to avoid the default generators. The generator indices are listed
 * based on the number of elements each index hashes. The following conditions must be met:
 *
 * +-----------+-------------------------------+----------------------+
 * | Hash size | Number of elements hashed (n) | Condition to use     |
 * |-----------+-------------------------------+----------------------|
 * | LOW       | n ≤ 8                         | 0 < hash_index ≤ 32  |
 * | MID       | 8 < n ≤ 16                    | 32 < hash_index ≤ 40 |
 * | HIGH      | 16 < n ≤ 44                   | 40 < hash_index ≤ 44 |
 * +-----------+-------------------------------+----------------------+
 *
 */
enum GeneratorIndex {
    /**
     * Indices with size ≤ 8
     */
    COMMITMENT = 1,                // Size = 7 (unused)
    COMMITMENT_PLACEHOLDER,        // Size = 1 (unused), for omitting some elements of commitment when partially comm
    OUTER_COMMITMENT,              // Size = 2
    NULLIFIER_HASHED_PRIVATE_KEY,  // Size = 1 (unused)
    NULLIFIER,                     // Size = 4 (unused)
    INITIALISATION_NULLIFIER,      // Size = 2 (unused)
    OUTER_NULLIFIER,               // Size = 2
    PUBLIC_DATA_READ,              // Size = 2
    PUBLIC_DATA_UPDATE_REQUEST,    // Size = 3
    FUNCTION_DATA,                 // Size = 3
    FUNCTION_LEAF,                 // Size = 4
    CONTRACT_DEPLOYMENT_DATA,      // Size = 4
    CONSTRUCTOR,                   // Size = 3
    CONSTRUCTOR_ARGS,              // Size = 8
    CONTRACT_ADDRESS,              // Size = 4
    CONTRACT_LEAF,                 // Size = 3
    CALL_CONTEXT,                  // Size = 6
    CALL_STACK_ITEM,               // Size = 3
    CALL_STACK_ITEM_2,             // Size = ? (unused), // TODO see function where it's used for explanation
    L1_TO_L2_MESSAGE_SECRET,       // Size = 1 (wrongly used)
    L2_TO_L1_MSG,                  // Size = 2 (unused)
    TX_CONTEXT,                    // Size = 4
    PUBLIC_LEAF_INDEX,             // Size = 2 (unused)
    PUBLIC_DATA_LEAF,              // Size = ? (unused) // TODO what's the expected size? Assuming ≤ 8
    SIGNED_TX_REQUEST,             // Size = 7
    GLOBAL_VARIABLES,              // Size = 4
    PARTIAL_CONTRACT_ADDRESS,      // Size = 7
    /**
     * Indices with size ≤ 16
     */
    TX_REQUEST = 33,  // Size = 14
    /**
     * Indices with size ≤ 44
     */
    VK = 41,                        // Size = 35
    PRIVATE_CIRCUIT_PUBLIC_INPUTS,  // Size = 39
    PUBLIC_CIRCUIT_PUBLIC_INPUTS,   // Size = 32 (unused)
    FUNCTION_ARGS,                  // Size ≤ 40
};

enum StorageSlotGeneratorIndex {
    BASE_SLOT,
    MAPPING_SLOT,
    MAPPING_SLOT_PLACEHOLDER,
};

// Enumerate the hash_sub_indices which are used for committing to private state note preimages.
// Start from 1.
enum PrivateStateNoteGeneratorIndex {
    VALUE = 1,
    OWNER,
    CREATOR,
    SALT,
    NONCE,
    MEMO,
    IS_DUMMY,
};

enum PrivateStateType { PARTITIONED = 1, WHOLE };

}  // namespace aztec3