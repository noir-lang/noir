global ARGS_LENGTH: Field = 16;
global RETURN_VALUES_LENGTH: Field = 4;

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
global MAX_NEW_COMMITMENTS_PER_CALL: Field = 16;
global MAX_NEW_NULLIFIERS_PER_CALL: Field = 16;
global MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL: Field = 4;
global MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL: Field = 4;
global MAX_NEW_L2_TO_L1_MSGS_PER_CALL: Field = 2;
global MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL: Field = 16;
global MAX_PUBLIC_DATA_READS_PER_CALL: Field = 16;
global MAX_READ_REQUESTS_PER_CALL: Field = 32;
global MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL: Field = 1;

// "PER TRANSACTION" CONSTANTS
global MAX_NEW_COMMITMENTS_PER_TX: Field = 64;
global MAX_NEW_NULLIFIERS_PER_TX: Field = 64;
global MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX: Field = 8;
global MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX: Field = 8;
global MAX_NEW_L2_TO_L1_MSGS_PER_TX: Field = 2;
global MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX: Field = 16;
global MAX_PUBLIC_DATA_READS_PER_TX: Field = 16;
global MAX_NEW_CONTRACTS_PER_TX: Field = 1;
global MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX: Field = 4;
global MAX_READ_REQUESTS_PER_TX: Field = 128;
global MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX: Field = 4;
global NUM_ENCRYPTED_LOGS_HASHES_PER_TX: Field = 1;
global NUM_UNENCRYPTED_LOGS_HASHES_PER_TX: Field = 1;
// docs:end:constants

// ROLLUP CONTRACT CONSTANTS - constants used only in l1-contracts
global NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP: Field = 16;

// TREES RELATED CONSTANTS
global VK_TREE_HEIGHT: Field = 3;
global FUNCTION_TREE_HEIGHT: Field = 5;
global CONTRACT_TREE_HEIGHT: Field = 16;
global NOTE_HASH_TREE_HEIGHT: Field = 32;
global PUBLIC_DATA_TREE_HEIGHT: Field = 40;
global NULLIFIER_TREE_HEIGHT: Field = 20;
global L1_TO_L2_MSG_TREE_HEIGHT: Field = 16;
global ROLLUP_VK_TREE_HEIGHT: Field = 8;

// SUB-TREES RELATED CONSTANTS
global CONTRACT_SUBTREE_HEIGHT: Field = 0;
global CONTRACT_SUBTREE_SIBLING_PATH_LENGTH: Field = 16;
global NOTE_HASH_SUBTREE_HEIGHT: Field = 6;
global NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH: Field = 26;
global NULLIFIER_SUBTREE_HEIGHT: Field = 6;
global PUBLIC_DATA_SUBTREE_HEIGHT: Field = 4;
global ARCHIVE_HEIGHT: Field = 16;
global NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH: Field = 14;
global PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH: Field = 36;
global L1_TO_L2_MSG_SUBTREE_HEIGHT: Field = 4;
global L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH: Field = 12;

// MISC CONSTANTS
global FUNCTION_SELECTOR_NUM_BYTES: Field = 4;
global MAPPING_SLOT_PEDERSEN_SEPARATOR: Field = 4;
// sha256 hash is stored in two fields to accommodate all 256-bits of the hash
global NUM_FIELDS_PER_SHA256: Field = 2;
global ARGS_HASH_CHUNK_LENGTH: u32 = 32;
global ARGS_HASH_CHUNK_COUNT: u32 = 16;

// NOIR CONSTANTS - constants used only in yarn-packages/noir-contracts
// Some are defined here because Noir doesn't yet support globals referencing other globals yet.
// Move these constants to a noir file once the issue bellow is resolved:
// https://github.com/noir-lang/noir/issues/1734
global L1_TO_L2_MESSAGE_LENGTH: Field = 8;
global L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH: Field = 25;
global MAX_NOTE_FIELDS_LENGTH: Field = 20;
// GET_NOTE_ORACLE_RETURN_LENGT = MAX_NOTE_FIELDS_LENGTH + 1 + 2
// The plus 1 is 1 extra field for nonce.
// + 2 for EXTRA_DATA: [number_of_return_notes, contract_address]
global GET_NOTE_ORACLE_RETURN_LENGTH: Field = 23;
global MAX_NOTES_PER_PAGE: Field = 10;
// VIEW_NOTE_ORACLE_RETURN_LENGTH = MAX_NOTES_PER_PAGE * (MAX_NOTE_FIELDS_LENGTH + 1) + 2;
global VIEW_NOTE_ORACLE_RETURN_LENGTH: Field = 212;
global CALL_CONTEXT_LENGTH: Field = 8;
global BLOCK_HEADER_LENGTH: Field = 7;
global FUNCTION_DATA_LENGTH: Field = 4;
global CONTRACT_DEPLOYMENT_DATA_LENGTH: Field = 6;
// Change this ONLY if you have changed the PrivateCircuitPublicInputs structure.
// In other words, if the structure/size of the public inputs of a function call changes then we
// should change this constant as well as the offsets in private_call_stack_item.nr
global PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH: Field = 189;
global CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH: Field = 3;
global CONTRACT_STORAGE_READ_LENGTH: Field = 2;
// Change this ONLY if you have changed the PublicCircuitPublicInputs structure.
global PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH: Field = 190;
global GET_NOTES_ORACLE_RETURN_LENGTH: Field = 674;
global CALL_PRIVATE_FUNCTION_RETURN_SIZE: Field = 199;
global PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH: Field = 87;
global PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH: Field = 177;
global COMMITMENTS_NUM_BYTES_PER_BASE_ROLLUP: Field = 2048;
global NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP: Field = 2048;
global PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP: Field = 1024;
global CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP: Field = 32;
global CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;
global CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED: Field = 52;
global L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;
global LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;

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
// Indices with size ≤ 8
global GENERATOR_INDEX__COMMITMENT = 1;
global GENERATOR_INDEX__COMMITMENT_NONCE = 2;
global GENERATOR_INDEX__UNIQUE_COMMITMENT = 3;
global GENERATOR_INDEX__SILOED_COMMITMENT = 4;
global GENERATOR_INDEX__NULLIFIER = 5;
global GENERATOR_INDEX__INITIALIZATION_NULLIFIER = 6;
global GENERATOR_INDEX__OUTER_NULLIFIER = 7;
global GENERATOR_INDEX__PUBLIC_DATA_READ = 8;
global GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST = 9;
global GENERATOR_INDEX__FUNCTION_DATA = 10;
global GENERATOR_INDEX__FUNCTION_LEAF = 11;
global GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA = 12;
global GENERATOR_INDEX__CONSTRUCTOR = 13;
global GENERATOR_INDEX__CONSTRUCTOR_ARGS = 14;
global GENERATOR_INDEX__CONTRACT_ADDRESS = 15;
global GENERATOR_INDEX__CONTRACT_LEAF = 16;
global GENERATOR_INDEX__CALL_CONTEXT = 17;
global GENERATOR_INDEX__CALL_STACK_ITEM = 18;
global GENERATOR_INDEX__CALL_STACK_ITEM_2 = 19;
global GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET = 20;
global GENERATOR_INDEX__L2_TO_L1_MSG = 21;
global GENERATOR_INDEX__TX_CONTEXT = 22;
global GENERATOR_INDEX__PUBLIC_LEAF_INDEX = 23;
global GENERATOR_INDEX__PUBLIC_DATA_LEAF = 24;
global GENERATOR_INDEX__SIGNED_TX_REQUEST = 25;
global GENERATOR_INDEX__GLOBAL_VARIABLES = 26;
global GENERATOR_INDEX__PARTIAL_ADDRESS = 27;
global GENERATOR_INDEX__BLOCK_HASH = 28;
global GENERATOR_INDEX__SIDE_EFFECT = 29;
// Indices with size ≤ 16
global GENERATOR_INDEX__TX_REQUEST = 33;
global GENERATOR_INDEX__SIGNATURE_PAYLOAD = 34;
// Indices with size ≤ 44
global GENERATOR_INDEX__VK = 41;
global GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS = 42;
global GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS = 43;
global GENERATOR_INDEX__FUNCTION_ARGS = 44;
