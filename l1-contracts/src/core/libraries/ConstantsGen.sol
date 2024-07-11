// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants in circuits.js
// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Constants Library
 * @author Aztec Labs
 * @notice Library that contains constants used throughout the Aztec protocol
 */
library Constants {
  // Prime field modulus
  uint256 internal constant P =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;
  uint256 internal constant MAX_FIELD_VALUE = P - 1;

  uint256 internal constant ARGS_LENGTH = 16;
  uint256 internal constant MAX_NOTE_HASHES_PER_CALL = 16;
  uint256 internal constant MAX_NULLIFIERS_PER_CALL = 16;
  uint256 internal constant MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL = 4;
  uint256 internal constant MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL = 16;
  uint256 internal constant MAX_L2_TO_L1_MSGS_PER_CALL = 2;
  uint256 internal constant MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL = 32;
  uint256 internal constant MAX_PUBLIC_DATA_READS_PER_CALL = 32;
  uint256 internal constant MAX_NOTE_HASH_READ_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_NULLIFIER_READ_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_KEY_VALIDATION_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_NOTE_ENCRYPTED_LOGS_PER_CALL = 16;
  uint256 internal constant MAX_ENCRYPTED_LOGS_PER_CALL = 4;
  uint256 internal constant MAX_UNENCRYPTED_LOGS_PER_CALL = 4;
  uint256 internal constant MAX_NOTE_HASHES_PER_TX = 64;
  uint256 internal constant MAX_NULLIFIERS_PER_TX = 64;
  uint256 internal constant MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX = 8;
  uint256 internal constant MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX = 32;
  uint256 internal constant MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 63;
  uint256 internal constant PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 1;
  uint256 internal constant MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_PUBLIC_DATA_READS_PER_TX = 64;
  uint256 internal constant MAX_L2_TO_L1_MSGS_PER_TX = 8;
  uint256 internal constant MAX_NOTE_HASH_READ_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_NULLIFIER_READ_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_KEY_VALIDATION_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_NOTE_ENCRYPTED_LOGS_PER_TX = 64;
  uint256 internal constant MAX_ENCRYPTED_LOGS_PER_TX = 8;
  uint256 internal constant MAX_UNENCRYPTED_LOGS_PER_TX = 8;
  uint256 internal constant MAX_PUBLIC_DATA_HINTS = 128;
  uint256 internal constant NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP = 16;
  uint256 internal constant VK_TREE_HEIGHT = 5;
  uint256 internal constant FUNCTION_TREE_HEIGHT = 5;
  uint256 internal constant NOTE_HASH_TREE_HEIGHT = 32;
  uint256 internal constant PUBLIC_DATA_TREE_HEIGHT = 40;
  uint256 internal constant NULLIFIER_TREE_HEIGHT = 20;
  uint256 internal constant L1_TO_L2_MSG_TREE_HEIGHT = 16;
  uint256 internal constant ARTIFACT_FUNCTION_TREE_MAX_HEIGHT = 5;
  uint256 internal constant NULLIFIER_TREE_ID = 0;
  uint256 internal constant NOTE_HASH_TREE_ID = 1;
  uint256 internal constant PUBLIC_DATA_TREE_ID = 2;
  uint256 internal constant L1_TO_L2_MESSAGE_TREE_ID = 3;
  uint256 internal constant ARCHIVE_TREE_ID = 4;
  uint256 internal constant NOTE_HASH_SUBTREE_HEIGHT = 6;
  uint256 internal constant NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH = 26;
  uint256 internal constant NULLIFIER_SUBTREE_HEIGHT = 6;
  uint256 internal constant PUBLIC_DATA_SUBTREE_HEIGHT = 6;
  uint256 internal constant ARCHIVE_HEIGHT = 16;
  uint256 internal constant NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH = 14;
  uint256 internal constant PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH = 34;
  uint256 internal constant L1_TO_L2_MSG_SUBTREE_HEIGHT = 4;
  uint256 internal constant L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH = 12;
  uint256 internal constant PRIVATE_KERNEL_INIT_INDEX = 0;
  uint256 internal constant PRIVATE_KERNEL_INNER_INDEX = 1;
  uint256 internal constant PRIVATE_KERNEL_RESET_FULL_INDEX = 2;
  uint256 internal constant PRIVATE_KERNEL_RESET_BIG_INDEX = 3;
  uint256 internal constant PRIVATE_KERNEL_RESET_MEDIUM_INDEX = 4;
  uint256 internal constant PRIVATE_KERNEL_RESET_SMALL_INDEX = 5;
  uint256 internal constant PRIVATE_KERNEL_TAIL_INDEX = 10;
  uint256 internal constant PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX = 11;
  uint256 internal constant EMPTY_NESTED_INDEX = 12;
  uint256 internal constant PRIVATE_KERNEL_EMPTY_INDEX = 13;
  uint256 internal constant PUBLIC_KERNEL_SETUP_INDEX = 14;
  uint256 internal constant PUBLIC_KERNEL_APP_LOGIC_INDEX = 15;
  uint256 internal constant PUBLIC_KERNEL_TEARDOWN_INDEX = 16;
  uint256 internal constant PUBLIC_KERNEL_TAIL_INDEX = 17;
  uint256 internal constant BASE_PARITY_INDEX = 18;
  uint256 internal constant ROOT_PARITY_INDEX = 19;
  uint256 internal constant BASE_ROLLUP_INDEX = 20;
  uint256 internal constant MERGE_ROLLUP_INDEX = 21;
  uint256 internal constant ROOT_ROLLUP_INDEX = 22;
  uint256 internal constant FUNCTION_SELECTOR_NUM_BYTES = 4;
  uint256 internal constant ARGS_HASH_CHUNK_LENGTH = 16;
  uint256 internal constant ARGS_HASH_CHUNK_COUNT = 16;
  uint256 internal constant MAX_ARGS_LENGTH = 256;
  uint256 internal constant INITIALIZATION_SLOT_SEPARATOR = 1000000000;
  uint256 internal constant INITIAL_L2_BLOCK_NUM = 1;
  uint256 internal constant BLOB_SIZE_IN_BYTES = 126976;
  uint256 internal constant MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS = 20000;
  uint256 internal constant MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS = 3000;
  uint256 internal constant MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS = 3000;
  uint256 internal constant REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS = 19;
  uint256 internal constant REGISTERER_UNCONSTRAINED_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS = 12;
  uint256 internal constant REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE =
    11121068431693264234253912047066709627593769337094408533543930778360;
  uint256 internal constant REGISTERER_PRIVATE_FUNCTION_BROADCASTED_MAGIC_VALUE =
    2889881020989534926461066592611988634597302675057895885580456197069;
  uint256 internal constant REGISTERER_UNCONSTRAINED_FUNCTION_BROADCASTED_MAGIC_VALUE =
    24399338136397901754495080759185489776044879232766421623673792970137;
  uint256 internal constant DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE =
    14061769416655647708490531650437236735160113654556896985372298487345;
  uint256 internal constant DEFAULT_GAS_LIMIT = 1000000000;
  uint256 internal constant DEFAULT_TEARDOWN_GAS_LIMIT = 100000000;
  uint256 internal constant DEFAULT_MAX_FEE_PER_GAS = 10;
  uint256 internal constant DEFAULT_INCLUSION_FEE = 0;
  uint256 internal constant DA_BYTES_PER_FIELD = 32;
  uint256 internal constant DA_GAS_PER_BYTE = 16;
  uint256 internal constant FIXED_DA_GAS = 512;
  uint256 internal constant FIXED_L2_GAS = 512;
  uint256 internal constant FIXED_AVM_STARTUP_L2_GAS = 1024;
  uint256 internal constant L2_GAS_PER_LOG_BYTE = 4;
  uint256 internal constant L2_GAS_PER_NOTE_HASH = 32;
  uint256 internal constant L2_GAS_PER_NULLIFIER = 64;
  uint256 internal constant CANONICAL_KEY_REGISTRY_ADDRESS =
    2153455745675440165069577621832684870696142028027528497509357256345838682961;
  uint256 internal constant CANONICAL_AUTH_REGISTRY_ADDRESS =
    18091885756106795278141309801070173692350235742979924147720536894670507925831;
  uint256 internal constant DEPLOYER_CONTRACT_ADDRESS =
    19511485909966796736993840362353440247573331327062358513665772226446629198132;
  uint256 internal constant REGISTERER_CONTRACT_ADDRESS =
    13402924717071282069537366635406026232165444473509746327951838324587448220160;
  uint256 internal constant GAS_TOKEN_ADDRESS =
    3159976153131520272419617514531889581796079438158800470341967144801191524489;
  uint256 internal constant AZTEC_ADDRESS_LENGTH = 1;
  uint256 internal constant GAS_FEES_LENGTH = 2;
  uint256 internal constant GAS_LENGTH = 2;
  uint256 internal constant GAS_SETTINGS_LENGTH = 7;
  uint256 internal constant CALL_CONTEXT_LENGTH = 5;
  uint256 internal constant CONTENT_COMMITMENT_LENGTH = 4;
  uint256 internal constant CONTRACT_INSTANCE_LENGTH = 5;
  uint256 internal constant CONTRACT_STORAGE_READ_LENGTH = 3;
  uint256 internal constant CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH = 3;
  uint256 internal constant ETH_ADDRESS_LENGTH = 1;
  uint256 internal constant FUNCTION_DATA_LENGTH = 2;
  uint256 internal constant FUNCTION_LEAF_PREIMAGE_LENGTH = 5;
  uint256 internal constant GLOBAL_VARIABLES_LENGTH = 8;
  uint256 internal constant APPEND_ONLY_TREE_SNAPSHOT_LENGTH = 2;
  uint256 internal constant L1_TO_L2_MESSAGE_LENGTH = 6;
  uint256 internal constant L2_TO_L1_MESSAGE_LENGTH = 3;
  uint256 internal constant SCOPED_L2_TO_L1_MESSAGE_LENGTH = 4;
  uint256 internal constant MAX_BLOCK_NUMBER_LENGTH = 2;
  uint256 internal constant KEY_VALIDATION_REQUEST_LENGTH = 4;
  uint256 internal constant KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH = 5;
  uint256 internal constant SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH = 6;
  uint256 internal constant PARTIAL_STATE_REFERENCE_LENGTH = 6;
  uint256 internal constant READ_REQUEST_LENGTH = 2;
  uint256 internal constant LOG_HASH_LENGTH = 3;
  uint256 internal constant SCOPED_LOG_HASH_LENGTH = 4;
  uint256 internal constant ENCRYPTED_LOG_HASH_LENGTH = 4;
  uint256 internal constant SCOPED_ENCRYPTED_LOG_HASH_LENGTH = 5;
  uint256 internal constant NOTE_LOG_HASH_LENGTH = 4;
  uint256 internal constant NOTE_HASH_LENGTH = 2;
  uint256 internal constant SCOPED_NOTE_HASH_LENGTH = 3;
  uint256 internal constant NULLIFIER_LENGTH = 3;
  uint256 internal constant SCOPED_NULLIFIER_LENGTH = 4;
  uint256 internal constant CALLER_CONTEXT_LENGTH = 3;
  uint256 internal constant PRIVATE_CALL_REQUEST_LENGTH = 15;
  uint256 internal constant SCOPED_PRIVATE_CALL_REQUEST_LENGTH = 16;
  uint256 internal constant ROLLUP_VALIDATION_REQUESTS_LENGTH = 2;
  uint256 internal constant STATE_REFERENCE_LENGTH = 8;
  uint256 internal constant TX_CONTEXT_LENGTH = 9;
  uint256 internal constant TX_REQUEST_LENGTH = 13;
  uint256 internal constant TOTAL_FEES_LENGTH = 1;
  uint256 internal constant HEADER_LENGTH = 23;
  uint256 internal constant PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH = 444;
  uint256 internal constant PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH = 481;
  uint256 internal constant PRIVATE_CALL_STACK_ITEM_LENGTH = 447;
  uint256 internal constant PUBLIC_CONTEXT_INPUTS_LENGTH = 40;
  uint256 internal constant AGGREGATION_OBJECT_LENGTH = 16;
  uint256 internal constant SCOPED_READ_REQUEST_LEN = 3;
  uint256 internal constant PUBLIC_DATA_READ_LENGTH = 2;
  uint256 internal constant VALIDATION_REQUESTS_LENGTH = 1090;
  uint256 internal constant PUBLIC_DATA_UPDATE_REQUEST_LENGTH = 3;
  uint256 internal constant COMBINED_ACCUMULATED_DATA_LENGTH = 333;
  uint256 internal constant COMBINED_CONSTANT_DATA_LENGTH = 41;
  uint256 internal constant PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH = 15;
  uint256 internal constant CALL_REQUEST_LENGTH = 7;
  uint256 internal constant PRIVATE_ACCUMULATED_DATA_LENGTH = 1160;
  uint256 internal constant PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH = 2300;
  uint256 internal constant PUBLIC_ACCUMULATED_DATA_LENGTH = 983;
  uint256 internal constant PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH = 3323;
  uint256 internal constant KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH = 384;
  uint256 internal constant CONSTANT_ROLLUP_DATA_LENGTH = 11;
  uint256 internal constant BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH = 28;
  uint256 internal constant ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH = 9;
  uint256 internal constant GET_NOTES_ORACLE_RETURN_LENGTH = 674;
  uint256 internal constant NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP = 2048;
  uint256 internal constant NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP = 2048;
  uint256 internal constant PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP = 4096;
  uint256 internal constant CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP = 32;
  uint256 internal constant CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP = 64;
  uint256 internal constant CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED = 52;
  uint256 internal constant L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP = 256;
  uint256 internal constant LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP = 64;
  uint256 internal constant NUM_MSGS_PER_BASE_PARITY = 4;
  uint256 internal constant NUM_BASE_PARITY_PER_ROOT_PARITY = 4;
  uint256 internal constant RECURSIVE_PROOF_LENGTH = 393;
  uint256 internal constant NESTED_RECURSIVE_PROOF_LENGTH = 393;
  uint256 internal constant TUBE_PROOF_LENGTH = 393;
  uint256 internal constant VERIFICATION_KEY_LENGTH_IN_FIELDS = 103;
  uint256 internal constant SENDER_SELECTOR = 0;
  uint256 internal constant ADDRESS_SELECTOR = 1;
  uint256 internal constant STORAGE_ADDRESS_SELECTOR = 1;
  uint256 internal constant FUNCTION_SELECTOR_SELECTOR = 2;
  uint256 internal constant START_GLOBAL_VARIABLES = 28;
  uint256 internal constant CHAIN_ID_SELECTOR = 28;
  uint256 internal constant VERSION_SELECTOR = 29;
  uint256 internal constant BLOCK_NUMBER_SELECTOR = 30;
  uint256 internal constant TIMESTAMP_SELECTOR = 31;
  uint256 internal constant COINBASE_SELECTOR = 32;
  uint256 internal constant UNUSED_FEE_RECIPIENT_SELECTOR = 33;
  uint256 internal constant FEE_PER_DA_GAS_SELECTOR = 34;
  uint256 internal constant FEE_PER_L2_GAS_SELECTOR = 35;
  uint256 internal constant END_GLOBAL_VARIABLES = 36;
  uint256 internal constant START_SIDE_EFFECT_COUNTER = 36;
  uint256 internal constant TRANSACTION_FEE_SELECTOR = 39;
  uint256 internal constant START_NOTE_HASH_EXISTS_WRITE_OFFSET = 0;
  uint256 internal constant START_NULLIFIER_EXISTS_OFFSET = 16;
  uint256 internal constant START_NULLIFIER_NON_EXISTS_OFFSET = 32;
  uint256 internal constant START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET = 48;
  uint256 internal constant START_SSTORE_WRITE_OFFSET = 64;
  uint256 internal constant START_SLOAD_WRITE_OFFSET = 96;
  uint256 internal constant START_EMIT_NOTE_HASH_WRITE_OFFSET = 128;
  uint256 internal constant START_EMIT_NULLIFIER_WRITE_OFFSET = 144;
  uint256 internal constant START_EMIT_L2_TO_L1_MSG_WRITE_OFFSET = 160;
  uint256 internal constant START_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET = 162;
}
