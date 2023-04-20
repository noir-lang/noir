#pragma once
#include <stddef.h>

namespace aztec3 {

// Note: must be kept in sync with ts/structs/constants.ts
constexpr size_t ARGS_LENGTH = 8;
constexpr size_t RETURN_VALUES_LENGTH = 4;
constexpr size_t EMITTED_EVENTS_LENGTH = 4;

constexpr size_t NEW_COMMITMENTS_LENGTH = 4;
constexpr size_t NEW_NULLIFIERS_LENGTH = 4;

constexpr size_t STATE_TRANSITIONS_LENGTH = 4;
constexpr size_t STATE_READS_LENGTH = 4;

constexpr size_t PRIVATE_CALL_STACK_LENGTH = 4;
constexpr size_t PUBLIC_CALL_STACK_LENGTH = 4;
constexpr size_t L1_MSG_STACK_LENGTH = 2;

constexpr size_t KERNEL_NEW_COMMITMENTS_LENGTH = 4;
constexpr size_t KERNEL_NEW_NULLIFIERS_LENGTH = 4;
constexpr size_t KERNEL_NEW_CONTRACTS_LENGTH = 1;
constexpr size_t KERNEL_PRIVATE_CALL_STACK_LENGTH = 8;
constexpr size_t KERNEL_PUBLIC_CALL_STACK_LENGTH = 8;
constexpr size_t KERNEL_L1_MSG_STACK_LENGTH = 4;
constexpr size_t KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH = 4;

constexpr size_t VK_TREE_HEIGHT = 3;
constexpr size_t FUNCTION_TREE_HEIGHT = 4;
constexpr size_t CONTRACT_TREE_HEIGHT = 4;
constexpr size_t PRIVATE_DATA_TREE_HEIGHT = 8;
constexpr size_t NULLIFIER_TREE_HEIGHT = 8;
constexpr size_t PUBLIC_DATA_TREE_HEIGHT = 32;

constexpr size_t CONTRACT_SUBTREE_DEPTH = 1;
constexpr size_t CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH = CONTRACT_TREE_HEIGHT - CONTRACT_SUBTREE_DEPTH;

constexpr size_t PRIVATE_DATA_SUBTREE_DEPTH = 3;
constexpr size_t PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH = NULLIFIER_TREE_HEIGHT - PRIVATE_DATA_SUBTREE_DEPTH;

constexpr size_t NULLIFIER_SUBTREE_DEPTH = 3;
constexpr size_t NULLIFIER_SUBTREE_INCLUSION_CHECK_DEPTH = NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_DEPTH;

constexpr size_t PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT = 8;
constexpr size_t CONTRACT_TREE_ROOTS_TREE_HEIGHT = 8;
constexpr size_t ROLLUP_VK_TREE_HEIGHT = 8; // TODO: update

constexpr size_t FUNCTION_SELECTOR_NUM_BYTES = 31; // must be <= 31

// Enumerate the hash_indices which are used for pedersen hashing
// Start from 1 to avoid the default generators.
enum GeneratorIndex {
    COMMITMENT = 1,
    COMMITMENT_PLACEHOLDER, // for omitting some elements of the commitment when partially committing.
    OUTER_COMMITMENT,
    NULLIFIER_HASHED_PRIVATE_KEY,
    NULLIFIER,
    INITIALISATION_NULLIFIER,
    OUTER_NULLIFIER,
    STATE_READ,
    STATE_TRANSITION,
    VK,
    FUNCTION_DATA,
    FUNCTION_LEAF,
    CONTRACT_DEPLOYMENT_DATA,
    CONSTRUCTOR,
    CONSTRUCTOR_ARGS,
    CONTRACT_ADDRESS,
    CONTRACT_LEAF,
    CALL_CONTEXT,
    CALL_STACK_ITEM,
    CALL_STACK_ITEM_2, // see function where it's used for explanation
    L1_MSG_STACK_ITEM,
    PRIVATE_CIRCUIT_PUBLIC_INPUTS,
    PUBLIC_CIRCUIT_PUBLIC_INPUTS,
    TX_CONTEXT,
    TX_REQUEST,
    PUBLIC_LEAF_INDEX,
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

} // namespace aztec3