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

// docs:start:constants
// "PER CALL" CONSTANTS
constexpr size_t MAX_NEW_COMMITMENTS_PER_CALL = 4;
constexpr size_t MAX_NEW_NULLIFIERS_PER_CALL = 4;
constexpr size_t MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL = 4;
constexpr size_t MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL = 4;
constexpr size_t MAX_NEW_L2_TO_L1_MSGS_PER_CALL = 2;
constexpr size_t MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL = 16;
constexpr size_t MAX_PUBLIC_DATA_READS_PER_CALL = 16;
constexpr size_t MAX_READ_REQUESTS_PER_CALL = 4;


// "PER TRANSACTION" CONSTANTS
constexpr size_t MAX_NEW_COMMITMENTS_PER_TX = MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL * MAX_NEW_COMMITMENTS_PER_CALL;
constexpr size_t MAX_NEW_NULLIFIERS_PER_TX = MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL * MAX_NEW_NULLIFIERS_PER_CALL;
constexpr size_t MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX = 8;
constexpr size_t MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX = 8;
constexpr size_t MAX_NEW_L2_TO_L1_MSGS_PER_TX = 2;
constexpr size_t MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 16;
constexpr size_t MAX_PUBLIC_DATA_READS_PER_TX = 16;
constexpr size_t MAX_NEW_CONTRACTS_PER_TX = 1;
constexpr size_t MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX = 4;
constexpr size_t MAX_READ_REQUESTS_PER_TX = MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL * MAX_READ_REQUESTS_PER_CALL;
constexpr size_t NUM_ENCRYPTED_LOGS_HASHES_PER_TX = 1;
constexpr size_t NUM_UNENCRYPTED_LOGS_HASHES_PER_TX = 1;
// docs:end:constants

////////////////////////////////////////////////////////////////////////////////
// ROLLUP CONTRACT CONSTANTS - constants used only in l1-contracts
////////////////////////////////////////////////////////////////////////////////
constexpr size_t NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP = 16;
// TODO(961): Use this constant everywhere instead of hard-coded "2".
constexpr size_t KERNELS_PER_BASE_ROLLUP = 2;
constexpr size_t COMMITMENTS_NUM_BYTES_PER_BASE_ROLLUP = KERNELS_PER_BASE_ROLLUP * MAX_NEW_COMMITMENTS_PER_TX * 32;
constexpr size_t NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP = KERNELS_PER_BASE_ROLLUP * MAX_NEW_NULLIFIERS_PER_TX * 32;
constexpr size_t PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 64;  // old value, new value
constexpr size_t CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP = KERNELS_PER_BASE_ROLLUP * MAX_NEW_CONTRACTS_PER_TX * 32;
constexpr size_t CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * MAX_NEW_CONTRACTS_PER_TX * 64;  // aztec address + eth address (padded to 0x20)
constexpr size_t CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED =
    KERNELS_PER_BASE_ROLLUP * MAX_NEW_CONTRACTS_PER_TX *
    52;  // same as prev except doesn't pad eth address. So 0x20 (aztec address) + 0x14 (eth address)
constexpr size_t L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP = KERNELS_PER_BASE_ROLLUP * MAX_NEW_L2_TO_L1_MSGS_PER_TX * 32;
constexpr size_t LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * 2 * 32;  // 1 for encrypted + 1 for unencrypted


// TREES RELATED CONSTANTS
constexpr size_t VK_TREE_HEIGHT = 3;
constexpr size_t FUNCTION_TREE_HEIGHT = 4;
constexpr size_t CONTRACT_TREE_HEIGHT = 16;
constexpr size_t PRIVATE_DATA_TREE_HEIGHT = 32;
constexpr size_t PUBLIC_DATA_TREE_HEIGHT = 254;
constexpr size_t NULLIFIER_TREE_HEIGHT = 16;
constexpr size_t L1_TO_L2_MSG_TREE_HEIGHT = 16;
constexpr size_t HISTORIC_BLOCKS_TREE_HEIGHT = 16;
constexpr size_t ROLLUP_VK_TREE_HEIGHT = 8;  // TODO: update


// SUB-TREES RELATED CONSTANTS
constexpr size_t CONTRACT_SUBTREE_HEIGHT = 1;
constexpr size_t CONTRACT_SUBTREE_SIBLING_PATH_LENGTH = CONTRACT_TREE_HEIGHT - CONTRACT_SUBTREE_HEIGHT;
constexpr size_t PRIVATE_DATA_SUBTREE_HEIGHT =
    static_cast<size_t>(log2(KERNELS_PER_BASE_ROLLUP * MAX_NEW_COMMITMENTS_PER_TX));
constexpr size_t PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH = PRIVATE_DATA_TREE_HEIGHT - PRIVATE_DATA_SUBTREE_HEIGHT;
constexpr size_t NULLIFIER_SUBTREE_HEIGHT =
    static_cast<size_t>(log2(KERNELS_PER_BASE_ROLLUP * MAX_NEW_NULLIFIERS_PER_TX));
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
 * | HIGH      | 16 < n ≤ 48                   | 40 < hash_index ≤ 48 |
 * +-----------+-------------------------------+----------------------+
 *
 * Note: When modifying, modify `GeneratorIndexPacker` in packer.hpp accordingly.
 */
enum GeneratorIndex {
    /**
     * Indices with size ≤ 8
     */
    COMMITMENT = 1,              // Size = 7 (unused)
    COMMITMENT_NONCE,            // Size = 2
    UNIQUE_COMMITMENT,           // Size = 2
    SILOED_COMMITMENT,           // Size = 2
    NULLIFIER,                   // Size = 4 (unused)
    INITIALISATION_NULLIFIER,    // Size = 2 (unused)
    OUTER_NULLIFIER,             // Size = 2
    PUBLIC_DATA_READ,            // Size = 2
    PUBLIC_DATA_UPDATE_REQUEST,  // Size = 3
    FUNCTION_DATA,               // Size = 4
    FUNCTION_LEAF,               // Size = 5
    CONTRACT_DEPLOYMENT_DATA,    // Size = 4
    CONSTRUCTOR,                 // Size = 3
    CONSTRUCTOR_ARGS,            // Size = 8
    CONTRACT_ADDRESS,            // Size = 4
    CONTRACT_LEAF,               // Size = 3
    CALL_CONTEXT,                // Size = 6
    CALL_STACK_ITEM,             // Size = 3
    CALL_STACK_ITEM_2,           // Size = ? (unused), // TODO see function where it's used for explanation
    L1_TO_L2_MESSAGE_SECRET,     // Size = 1
    L2_TO_L1_MSG,                // Size = 2 (unused)
    TX_CONTEXT,                  // Size = 4
    PUBLIC_LEAF_INDEX,           // Size = 2 (unused)
    PUBLIC_DATA_LEAF,            // Size = ? (unused) // TODO what's the expected size? Assuming ≤ 8
    SIGNED_TX_REQUEST,           // Size = 7
    GLOBAL_VARIABLES,            // Size = 4
    PARTIAL_ADDRESS,             // Size = 7
    BLOCK_HASH,                  // Size = 6
    /**
     * Indices with size ≤ 16
     */
    TX_REQUEST = 33,    // Size = 14
    SIGNATURE_PAYLOAD,  // Size = 13
    /**
     * Indices with size ≤ 44
     */
    VK = 41,                        // Size = 35
    PRIVATE_CIRCUIT_PUBLIC_INPUTS,  // Size = 45
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

////////////////////////////////////////////////////////////////////////////////
// NOIR CONSTANTS - constants used only in yarn-packages/noir-contracts
// --> Here because Noir doesn't yet support globals referencing other globals yet and doing so in C++ seems to be the
// best thing to do for now. Move these constants to a noir file once the issue bellow is resolved:
// https://github.com/noir-lang/noir/issues/1734
constexpr size_t L1_TO_L2_MESSAGE_LENGTH = 8;
// message length + sibling path (same size as tree height) + 1 field for root + 1 field for index
constexpr size_t L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH = L1_TO_L2_MESSAGE_LENGTH + L1_TO_L2_MSG_TREE_HEIGHT + 1 + 1;

// TODO: Remove these when nested array is supported.
constexpr size_t MAX_NOTE_FIELDS_LENGTH = 20;
// MAX_NOTE_FIELDS_LENGTH + 1: the plus 1 is 1 extra field for nonce.
// + 2 for EXTRA_DATA: [number_of_return_notes, contract_address]
constexpr size_t GET_NOTE_ORACLE_RETURN_LENGTH = MAX_NOTE_FIELDS_LENGTH + 1 + 2;
constexpr size_t GET_NOTES_ORACLE_RETURN_LENGTH = MAX_READ_REQUESTS_PER_CALL * (MAX_NOTE_FIELDS_LENGTH + 1) + 2;
constexpr size_t MAX_NOTES_PER_PAGE = 10;
// + 2 for EXTRA_DATA: [number_of_return_notes, contract_address]
constexpr size_t VIEW_NOTE_ORACLE_RETURN_LENGTH = MAX_NOTES_PER_PAGE * (MAX_NOTE_FIELDS_LENGTH + 1) + 2;

constexpr size_t CALL_CONTEXT_LENGTH = 6;
// Must be updated if any data is added into the block hash calculation.
constexpr size_t HISTORIC_BLOCK_DATA_LENGTH = 7;
constexpr size_t FUNCTION_DATA_LENGTH = 4;
constexpr size_t CONTRACT_DEPLOYMENT_DATA_LENGTH = 6;

// Change this ONLY if you have changed the PrivateCircuitPublicInputs structure in C++.
// In other words, if the structure/size of the public inputs of a function call changes then we
// should change this constant as well as the offsets in private_call_stack_item.nr
constexpr size_t PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH =
    CALL_CONTEXT_LENGTH + 1  // +1 for args_hash
    + RETURN_VALUES_LENGTH + MAX_READ_REQUESTS_PER_CALL + MAX_NEW_COMMITMENTS_PER_CALL +
    2 * MAX_NEW_NULLIFIERS_PER_CALL + MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL + MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL +
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL + NUM_FIELDS_PER_SHA256 + NUM_FIELDS_PER_SHA256 + 2  // + 2 for logs preimage lengths
    + HISTORIC_BLOCK_DATA_LENGTH + CONTRACT_DEPLOYMENT_DATA_LENGTH + 2;                 // + 2 for chain_id and version

constexpr size_t PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH =
    1 + 1  // call_context_hash + args_hash
    + RETURN_VALUES_LENGTH + MAX_READ_REQUESTS_PER_CALL + MAX_NEW_COMMITMENTS_PER_CALL +
    2 * MAX_NEW_NULLIFIERS_PER_CALL + MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL + MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL +
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL + NUM_FIELDS_PER_SHA256 + NUM_FIELDS_PER_SHA256 + 2  // + 2 for logs preimage lengths
    + HISTORIC_BLOCK_DATA_LENGTH + 3;  // + 3 for contract_deployment_data.hash(), chain_id, version

constexpr size_t CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH = 3;
constexpr size_t CONTRACT_STORAGE_READ_LENGTH = 2;

// Change this ONLY if you have changed the PublicCircuitPublicInputs structure in C++.
constexpr size_t PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH =
    CALL_CONTEXT_LENGTH + 1 + RETURN_VALUES_LENGTH +  // + 1 for args_hash
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL * CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH +
    MAX_PUBLIC_DATA_READS_PER_CALL * CONTRACT_STORAGE_READ_LENGTH + MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL +
    MAX_NEW_COMMITMENTS_PER_CALL + MAX_NEW_NULLIFIERS_PER_CALL + MAX_NEW_L2_TO_L1_MSGS_PER_CALL +
    NUM_FIELDS_PER_SHA256 + 1 +      // + 1 for unencrypted logs preimage length
    HISTORIC_BLOCK_DATA_LENGTH + 2;  // + 2 for chain_id and version

constexpr size_t PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH =
    2 + RETURN_VALUES_LENGTH +  // + 1 for args_hash + 1 call_context.hash
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL + MAX_PUBLIC_DATA_READS_PER_CALL + MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL +
    MAX_NEW_COMMITMENTS_PER_CALL + MAX_NEW_NULLIFIERS_PER_CALL + MAX_NEW_L2_TO_L1_MSGS_PER_CALL +
    NUM_FIELDS_PER_SHA256 +          // unencrypted_logs_hash (being represented by NUM_FIELDS_PER_SHA256)
    HISTORIC_BLOCK_DATA_LENGTH + 2;  // unencrypted_log_preimages_length + prover_address


// Size of the return value of a private function call,
constexpr size_t CALL_PRIVATE_FUNCTION_RETURN_SIZE =
    1 + FUNCTION_DATA_LENGTH + PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH + 1;

constexpr size_t EMPTY_NULLIFIED_COMMITMENT = 1000000;

}  // namespace aztec3