global ARGS_LENGTH: u64 = 16;
global RETURN_VALUES_LENGTH: u64 = 4;

/**
 * Convention for constant array lengths are mainly divided in 2 classes:
 *  - FUNCTION CALL
 *  - TRANSACTION
 *
 * Agreed convention is to use MAX_XXX_PER_CALL resp. MAX_XXX_PER_TX, where XXX denotes a type of element such as
 * commitment, or nullifier, e.g.,:
 *  - MAX_NEW_NULLIFIERS_PER_CALL
 *  - MAX_NEW_NOTE_HASHES_PER_TX
 *
 * In the kernel circuits, we accumulate elements such as note hashes and the nullifiers from all functions calls in a
 * transaction. Therefore, we always must have:
 * MAX_XXX_PER_TX ≥ MAX_XXX_PER_CALL
 *
 * For instance:
 * MAX_NEW_NOTE_HASHES_PER_TX ≥ MAX_NEW_NOTE_HASHES_PER_CALL
 * MAX_NEW_NULLIFIERS_PER_TX ≥ MAX_NEW_NULLIFIERS_PER_CALL
 *
 */

// docs:start:constants
// "PER CALL" CONSTANTS
global MAX_NEW_NOTE_HASHES_PER_CALL: u64 = 16;
global MAX_NEW_NULLIFIERS_PER_CALL: u64 = 16;
global MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL: u64 = 4;
global MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL: u64 = 4;
global MAX_NEW_L2_TO_L1_MSGS_PER_CALL: u64 = 2;
global MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL: u64 = 16;
global MAX_PUBLIC_DATA_READS_PER_CALL: u64 = 16;
global MAX_NOTE_HASH_READ_REQUESTS_PER_CALL: u64 = 32;
global MAX_NULLIFIER_READ_REQUESTS_PER_CALL: u64 = 2; // Change it to a larger value when there's a seperate reset circuit.
global MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL: u64 = 2;
global MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL: u64 = 1;

// "PER TRANSACTION" CONSTANTS
global MAX_NEW_NOTE_HASHES_PER_TX: u64 = 64;
global MAX_NEW_NULLIFIERS_PER_TX: u64 = 64;
global MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX: u64 = 8;
global MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX: u64 = 8;
global MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX: u64 = 32;
global MAX_PUBLIC_DATA_READS_PER_TX: u64 = 32;
global MAX_NEW_L2_TO_L1_MSGS_PER_TX: u64 = 2;
global MAX_NOTE_HASH_READ_REQUESTS_PER_TX: u64 = 128;
global MAX_NULLIFIER_READ_REQUESTS_PER_TX: u64 = 8; // Change it to a larger value when there's a seperate reset circuit.
global MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX: u64 = 8;
global MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX: u64 = 4;
global NUM_ENCRYPTED_LOGS_HASHES_PER_TX: u64 = 1;
global NUM_UNENCRYPTED_LOGS_HASHES_PER_TX: u64 = 1;
// docs:end:constants

// ROLLUP CONTRACT CONSTANTS - constants used only in l1-contracts
global NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP: u64 = 16;

// TREES RELATED CONSTANTS
global VK_TREE_HEIGHT: u64 = 3;
global FUNCTION_TREE_HEIGHT: u64 = 5;
global NOTE_HASH_TREE_HEIGHT: u64 = 32;
global PUBLIC_DATA_TREE_HEIGHT: u64 = 40;
global NULLIFIER_TREE_HEIGHT: u64 = 20;
global L1_TO_L2_MSG_TREE_HEIGHT: u64 = 16;
global ROLLUP_VK_TREE_HEIGHT: u64 = 8;
global ARTIFACT_FUNCTION_TREE_MAX_HEIGHT = 5;
global NULLIFIER_TREE_ID = 0;
global NOTE_HASH_TREE_ID = 1;
global PUBLIC_DATA_TREE_ID = 2;
global L1_TO_L2_MESSAGE_TREE_ID = 3;
global ARCHIVE_TREE_ID = 4;

// SUB-TREES RELATED CONSTANTS
global NOTE_HASH_SUBTREE_HEIGHT: u64 = 6;
global NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH: u64 = 26;
global NULLIFIER_SUBTREE_HEIGHT: u64 = 6;
global PUBLIC_DATA_SUBTREE_HEIGHT: u64 = 5;
global ARCHIVE_HEIGHT: u64 = 16;
global NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH: u64 = 14;
global PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH: u64 = 35;
global L1_TO_L2_MSG_SUBTREE_HEIGHT: u64 = 4;
global L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH: u64 = 12;

// MISC CONSTANTS
global FUNCTION_SELECTOR_NUM_BYTES: Field = 4;
global ARGS_HASH_CHUNK_LENGTH: u64 = 64;
global ARGS_HASH_CHUNK_COUNT: u64 = 64;
global MAX_ARGS_LENGTH: u64 = ARGS_HASH_CHUNK_COUNT * ARGS_HASH_CHUNK_LENGTH;
// The following is used in immutable state variables to compute an initialization slot whose value is used to
// determine whether a given variable has been initialized (by asserting that the value in the slot is 0).
// The initialization slot is computed by adding the constant bellow to the variable's storage slot. This constant has
// to be large enough so that it's ensured that it doesn't collide with storage slots of other variables.
global INITIALIZATION_SLOT_SEPARATOR: Field = 1000_000_000;
global INITIAL_L2_BLOCK_NUM: Field = 1;
// 126976 = 31 * 4096;
global BLOB_SIZE_IN_BYTES: Field = 126976;

// CONTRACT CLASS CONSTANTS
global MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS: u64 = 15000;
// Bytecode size for private functions is per function, not for the entire contract.
// Note that private functions bytecode includes a mix of acir and brillig.
global MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS: u64 = 3000;
// Same for unconstrained functions: the size is per function.
global MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS: u64 = 3000;
// How many fields are on the serialized ClassPrivateFunctionBroadcasted event in addition to MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS.
global REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS: u64 = 19;
// How many fields are on the serialized ClassUnconstrainedFunctionBroadcasted event in addition to MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS.
global REGISTERER_UNCONSTRAINED_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS: u64 = 12;
// Since we are not yet emitting selectors we'll use this magic value to identify events emitted by the ClassRegisterer.
// This is just a stopgap until we implement proper selectors.
// sha224sum 'struct ContractClassRegistered {contract_class_id: ContractClassId, version: Field, artifact_hash: Field, private_functions_root: Field, packed_public_bytecode: [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS] }'
global REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE = 0x6999d1e02b08a447a463563453cb36919c9dd7150336fc7c4d2b52f8;
// sha224sum 'struct ClassPrivateFunctionBroadcasted'
global REGISTERER_PRIVATE_FUNCTION_BROADCASTED_MAGIC_VALUE = 0x1b70e95fde0b70adc30496b90a327af6a5e383e028e7a43211a07bcd;
// sha224sum 'struct ClassUnconstrainedFunctionBroadcasted'
global REGISTERER_UNCONSTRAINED_FUNCTION_BROADCASTED_MAGIC_VALUE = 0xe7af816635466f128568edb04c9fa024f6c87fb9010fdbffa68b3d99;

// CONTRACT INSTANCE CONSTANTS
// sha224sum 'struct ContractInstanceDeployed'
global DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE = 0x85864497636cf755ae7bde03f267ce01a520981c21c3682aaf82a631;
global DEPLOYER_CONTRACT_ADDRESS = 0x1ad693effc2a4a40fdf5911ff29e19d2b6cd3cf73e6c3685519a6fb783144dcb;

// NOIR CONSTANTS - constants used only in yarn-packages/noir-contracts
// Some are defined here because Noir doesn't yet support globals referencing other globals yet.
// Move these constants to a noir file once the issue below is resolved:
// https://github.com/noir-lang/noir/issues/1734
global L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH: u64 = 17;
global MAX_NOTE_FIELDS_LENGTH: u64 = 20;
// GET_NOTE_ORACLE_RETURN_LENGT = MAX_NOTE_FIELDS_LENGTH + 1 + 2
// The plus 1 is 1 extra field for nonce.
// + 2 for EXTRA_DATA: [number_of_return_notes, contract_address]
global GET_NOTE_ORACLE_RETURN_LENGTH: u64 = 23;
global MAX_NOTES_PER_PAGE: u64 = 10;
// VIEW_NOTE_ORACLE_RETURN_LENGTH = MAX_NOTES_PER_PAGE * (MAX_NOTE_FIELDS_LENGTH + 1) + 2;
global VIEW_NOTE_ORACLE_RETURN_LENGTH: u64 = 212;

// LENGTH OF STRUCTS SERIALIZED TO FIELDS
global AZTEC_ADDRESS_LENGTH = 1;
global CALL_CONTEXT_LENGTH: u64 = 7;
global CONTENT_COMMITMENT_LENGTH: u64 = 4;
global CONTRACT_INSTANCE_LENGTH: u64 = 6;
global CONTRACT_STORAGE_READ_LENGTH: u64 = 2;
global CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH: u64 = 2;
global ETH_ADDRESS_LENGTH = 1;
global FUNCTION_DATA_LENGTH: u64 = 2;
global FUNCTION_LEAF_PREIMAGE_LENGTH: u64 = 5;
global GLOBAL_VARIABLES_LENGTH: u64 = 6;
global HEADER_LENGTH: u64 = 20; // 2 for last_archive, 4 for content commitment, 8 for state reference, 6 for global vars
global L1_TO_L2_MESSAGE_LENGTH: u64 = 6;
global L2_TO_L1_MESSAGE_LENGTH: u64 = 2;
global MAX_BLOCK_NUMBER_LENGTH: u64 = 2; // 1 for the option flag, 1 for the value
global NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH = 4;
global NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_LENGTH = 5;
global PARTIAL_STATE_REFERENCE_LENGTH: u64 = 6;
global PRIVATE_CALL_STACK_ITEM_LENGTH: u64 = 210;
// Change this ONLY if you have changed the PrivateCircuitPublicInputs structure.
// In other words, if the structure/size of the public inputs of a function call changes then we should change this
// constant as well PRIVATE_CALL_STACK_ITEM_LENGTH
global PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH: u64 = 207;
// Change this ONLY if you have changed the PublicCircuitPublicInputs structure.
global PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH: u64 = 198;
global STATE_REFERENCE_LENGTH: u64 = 8; // 2 for snap + 8 for partial
global TX_CONTEXT_DATA_LENGTH: u64 = 4;
global TX_REQUEST_LENGTH: u64 = 8; // 2 + TX_CONTEXT_DATA_LENGTH + FUNCTION_DATA_LENGTH

global ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH: Field = 11; // 2 + FUNCTION_DATA_LENGTH + CALL_CONTEXT_LENGTH
global GET_NOTES_ORACLE_RETURN_LENGTH: u64 = 674;
global NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP: Field = 2048;
global NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP: Field = 2048;
global PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP: Field = 2048;
global CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP: Field = 32;
global CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;
global CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED: Field = 52;
global L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;
global LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;
global NUM_MSGS_PER_BASE_PARITY: u64 = 4;
// NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP / NUM_MSGS_PER_BASE_PARITY
global NUM_BASE_PARITY_PER_ROOT_PARITY: u64 = 4;

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
global GENERATOR_INDEX__NOTE_HASH = 1;
global GENERATOR_INDEX__NOTE_HASH_NONCE = 2;
global GENERATOR_INDEX__UNIQUE_NOTE_HASH = 3;
global GENERATOR_INDEX__SILOED_NOTE_HASH = 4;
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
global GENERATOR_INDEX__FEE_PAYLOAD = 30;
// Indices with size ≤ 16
global GENERATOR_INDEX__TX_REQUEST = 33;
global GENERATOR_INDEX__SIGNATURE_PAYLOAD = 34;
// Indices with size ≤ 44
global GENERATOR_INDEX__VK = 41;
global GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS = 42;
global GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS = 43;
// TODO: Function args generator index is being used to hash 64 items
global GENERATOR_INDEX__FUNCTION_ARGS = 44;
global GENERATOR_INDEX__AUTHWIT_INNER = 45;
global GENERATOR_INDEX__AUTHWIT_OUTER = 46;
