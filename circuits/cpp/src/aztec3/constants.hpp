#pragma once
#include <stddef.h>

// NOTE: When modifying names of constants or enums do the changes in `src/aztec3/circuits/abis/packers.hpp` as well

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

constexpr size_t ARGS_LENGTH = 16;
constexpr size_t RETURN_VALUES_LENGTH = 4;

/**
 * Convention for constant array lengths are mainly divided in 2 classes:
 *  - FUNCTION CALL
 *  - TRANSACTION
 *
 * Agreed convention is to use MAX_XXX_PER_CALL resp. MAX_XXX_PER_TX, where XXX denotes a type of element such as
 * commitment, or nullifier, e.g.,:
 *  - MAX_NEW_NULLIFIERS_PER_CALL
 *  - MAX_NEW_COMMITMENTS_PER_TX
 *
 * In the kernel circuits, we accumulate elements such as commitments and the nullifiers from all functions calls in a
 * transaction. Therefore, we always must have:
 * MAX_XXX_PER_TX ≥ MAX_XXX_PER_CALL
 *
 * For instance:
 * MAX_NEW_COMMITMENTS_PER_TX ≥ MAX_NEW_COMMITMENTS_PER_CALL
 * MAX_NEW_NULLIFIERS_PER_TX ≥ MAX_NEW_NULLIFIERS_PER_CALL
 *
 */

// "PER CALL" CONSTANTS
constexpr size_t MAX_NEW_COMMITMENTS_PER_CALL = 4;
constexpr size_t MAX_NEW_NULLIFIERS_PER_CALL = 4;
constexpr size_t MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL = 4;
constexpr size_t MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL = 4;
constexpr size_t MAX_NEW_L2_TO_L1_MSGS_PER_CALL = 2;
constexpr size_t MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL = 4;
constexpr size_t MAX_PUBLIC_DATA_READS_PER_CALL = 4;
constexpr size_t MAX_READ_REQUESTS_PER_CALL = 4;


// "PER TRANSACTION" CONSTANTS
constexpr size_t MAX_NEW_COMMITMENTS_PER_TX = MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL * MAX_NEW_COMMITMENTS_PER_CALL;
constexpr size_t MAX_NEW_NULLIFIERS_PER_TX = MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL * MAX_NEW_NULLIFIERS_PER_CALL;
constexpr size_t MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX = 8;
constexpr size_t MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX = 8;
constexpr size_t MAX_NEW_L2_TO_L1_MSGS_PER_TX = 2;
constexpr size_t MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 4;
constexpr size_t MAX_PUBLIC_DATA_READS_PER_TX = 4;
constexpr size_t MAX_NEW_CONTRACTS_PER_TX = 1;
constexpr size_t MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX = 4;
constexpr size_t MAX_READ_REQUESTS_PER_TX = MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL * MAX_READ_REQUESTS_PER_CALL;
constexpr size_t NUM_ENCRYPTED_LOGS_HASHES_PER_TX = 1;
constexpr size_t NUM_UNENCRYPTED_LOGS_HASHES_PER_TX = 1;


// ROLLUP CONSTANTS
constexpr size_t NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP = 16;
// TODO(961): Use this constant everywhere instead of hard-coded "2".
constexpr size_t KERNELS_PER_ROLLUP = 2;


// TREES RELATED CONSTANTS
constexpr size_t VK_TREE_HEIGHT = 3;
constexpr size_t FUNCTION_TREE_HEIGHT = 4;
constexpr size_t CONTRACT_TREE_HEIGHT = 16;
constexpr size_t PRIVATE_DATA_TREE_HEIGHT = 32;
constexpr size_t PUBLIC_DATA_TREE_HEIGHT = 254;
constexpr size_t NULLIFIER_TREE_HEIGHT = 16;
constexpr size_t L1_TO_L2_MSG_TREE_HEIGHT = 16;
constexpr size_t PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT = 16;
constexpr size_t CONTRACT_TREE_ROOTS_TREE_HEIGHT = 16;
constexpr size_t L1_TO_L2_MSG_TREE_ROOTS_TREE_HEIGHT = 16;
constexpr size_t ROLLUP_VK_TREE_HEIGHT = 8;  // TODO: update


// SUB-TREES RELATED CONSTANTS
constexpr size_t CONTRACT_SUBTREE_HEIGHT = 1;
constexpr size_t CONTRACT_SUBTREE_SIBLING_PATH_LENGTH = CONTRACT_TREE_HEIGHT - CONTRACT_SUBTREE_HEIGHT;
constexpr size_t PRIVATE_DATA_SUBTREE_HEIGHT =
    static_cast<size_t>(log2(KERNELS_PER_ROLLUP * MAX_NEW_COMMITMENTS_PER_TX));
constexpr size_t PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH = PRIVATE_DATA_TREE_HEIGHT - PRIVATE_DATA_SUBTREE_HEIGHT;
constexpr size_t NULLIFIER_SUBTREE_HEIGHT = static_cast<size_t>(log2(KERNELS_PER_ROLLUP * MAX_NEW_NULLIFIERS_PER_TX));
constexpr size_t NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH = NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT;
constexpr size_t L1_TO_L2_MSG_SUBTREE_HEIGHT = static_cast<size_t>(log2(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP));
constexpr size_t L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH = L1_TO_L2_MSG_TREE_HEIGHT - L1_TO_L2_MSG_SUBTREE_HEIGHT;


// MISC CONSTANTS
constexpr size_t FUNCTION_SELECTOR_NUM_BYTES = 4;  // must be <= 31
constexpr size_t MAPPING_SLOT_PEDERSEN_SEPARATOR = 4;
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
 * Note: When modifying, modify `GeneratorIndexPacker` in packer.hpp accordingly.
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

// Note: When modifying, modify `StorageSlotGeneratorIndexPacker` in packer.hpp accordingly.
enum StorageSlotGeneratorIndex {
    BASE_SLOT,
    MAPPING_SLOT,
    MAPPING_SLOT_PLACEHOLDER,
};

// Enumerate the hash_sub_indices which are used for committing to private state note preimages.
// Start from 1.
// Note: When modifying, modify `PrivateStateNoteGeneratorIndexPacker` in packer.hpp accordingly.
enum PrivateStateNoteGeneratorIndex {
    VALUE = 1,
    OWNER,
    CREATOR,
    SALT,
    NONCE,
    MEMO,
    IS_DUMMY,
};

// Note: When modifying, modify `PrivateStateTypePacker` in packer.hpp accordingly.
enum PrivateStateType { PARTITIONED = 1, WHOLE };

}  // namespace aztec3