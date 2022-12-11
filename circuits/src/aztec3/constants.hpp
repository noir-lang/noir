#pragma once
#include <stddef.h>

namespace aztec3 {

constexpr size_t CUSTOM_INPUTS_LENGTH = 8;
constexpr size_t CUSTOM_OUTPUTS_LENGTH = 4;
constexpr size_t EMITTED_EVENTS_LENGTH = 4;

constexpr size_t OUTPUT_COMMITMENTS_LENGTH = 4;
constexpr size_t INPUT_NULLIFIERS_LENGTH = 4;

constexpr size_t STATE_TRANSITIONS_LENGTH = 4;
constexpr size_t STATE_READS_LENGTH = 4;

constexpr size_t PRIVATE_CALL_STACK_LENGTH = 4;
constexpr size_t PUBLIC_CALL_STACK_LENGTH = 4;
constexpr size_t CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH = 2;
constexpr size_t PARTIAL_L1_CALL_STACK_LENGTH = 2;

constexpr size_t KERNEL_OUTPUT_COMMITMENTS_LENGTH = 16;
constexpr size_t KERNEL_INPUT_NULLIFIERS_LENGTH = 16;
constexpr size_t KERNEL_PRIVATE_CALL_STACK_LENGTH = 8;
constexpr size_t KERNEL_PUBLIC_CALL_STACK_LENGTH = 8;
constexpr size_t KERNEL_CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH = 4;
constexpr size_t KERNEL_L1_CALL_STACK_LENGTH = 4;
constexpr size_t KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH = 4;

constexpr size_t VK_TREE_HEIGHT = 3;
constexpr size_t CONTRACT_TREE_HEIGHT = 4;
constexpr size_t PRIVATE_DATA_TREE_HEIGHT = 8;
constexpr size_t NULLIFIER_TREE_HEIGHT = 8;

// Enumerate the hash_indices which are used for pedersen hashing
// Start from 1 to avoid the default generators.
enum GeneratorIndex {
    COMMITMENT,
    COMMITMENT_PLACEHOLDER, // for omitting some elements of the commitment when partially committing.
    OUTER_COMMITMENT,
    NULLIFIER_HASHED_PRIVATE_KEY,
    NULLIFIER,
    OUTER_NULLIFIER,
    STATE_READ,
    STATE_TRANSITION,
    CONTRACT_ADDRESS,
    FUNCTION_SIGNATURE,
    CALL_CONTEXT,
    CALL_STACK_ITEM,
    CALL_STACK_ITEM_2, // see function where it's used for explanation
    PARTIAL_L1_CALL_STACK_ITEM,
    L1_CALL_STACK_ITEM,
    PRIVATE_CIRCUIT_PUBLIC_INPUTS,
    PUBLIC_CIRCUIT_PUBLIC_INPUTS,
};

// Enumerate the hash_sub_indices which are used for committing to private state note preimages.
// Start from 1.
enum PrivateStateNoteGeneratorIndex {
    MAPPING_SLOT = 1,
    MAPPING_SLOT_PLACEHOLDER, // for omitting some mapping key values when partially committing.
    VALUE,
    OWNER,
    CREATOR,
    SALT,
    INPUT_NULLIFIER,
    MEMO,
    IS_REAL,
};

enum PrivateStateType { PARTITIONED = 1, WHOLE };

} // namespace aztec3