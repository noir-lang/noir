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
  uint256 internal constant MAX_NEW_NOTE_HASHES_PER_CALL = 16;
  uint256 internal constant MAX_NEW_NULLIFIERS_PER_CALL = 16;
  uint256 internal constant MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL = 4;
  uint256 internal constant MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL = 16;
  uint256 internal constant MAX_NEW_L2_TO_L1_MSGS_PER_CALL = 2;
  uint256 internal constant MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL = 32;
  uint256 internal constant MAX_PUBLIC_DATA_READS_PER_CALL = 16;
  uint256 internal constant MAX_NOTE_HASH_READ_REQUESTS_PER_CALL = 32;
  uint256 internal constant MAX_NULLIFIER_READ_REQUESTS_PER_CALL = 32;
  uint256 internal constant MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL = 32;
  uint256 internal constant MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_KEY_VALIDATION_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_NOTE_ENCRYPTED_LOGS_PER_CALL = 16;
  uint256 internal constant MAX_ENCRYPTED_LOGS_PER_CALL = 4;
  uint256 internal constant MAX_UNENCRYPTED_LOGS_PER_CALL = 4;
  uint256 internal constant MAX_NEW_NOTE_HASHES_PER_TX = 64;
  uint256 internal constant MAX_NEW_NULLIFIERS_PER_TX = 64;
  uint256 internal constant MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX = 8;
  uint256 internal constant MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX = 32;
  uint256 internal constant MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 63;
  uint256 internal constant PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 1;
  uint256 internal constant MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_PUBLIC_DATA_READS_PER_TX = 32;
  uint256 internal constant MAX_NEW_L2_TO_L1_MSGS_PER_TX = 8;
  uint256 internal constant MAX_NOTE_HASH_READ_REQUESTS_PER_TX = 128;
  uint256 internal constant MAX_NULLIFIER_READ_REQUESTS_PER_TX = 128;
  uint256 internal constant MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX = 128;
  uint256 internal constant MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_KEY_VALIDATION_REQUESTS_PER_TX = 64;
  uint256 internal constant MAX_NOTE_ENCRYPTED_LOGS_PER_TX = 64;
  uint256 internal constant MAX_ENCRYPTED_LOGS_PER_TX = 8;
  uint256 internal constant MAX_UNENCRYPTED_LOGS_PER_TX = 8;
  uint256 internal constant MAX_PUBLIC_DATA_HINTS = 64;
  uint256 internal constant NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP = 16;
  uint256 internal constant VK_TREE_HEIGHT = 3;
  uint256 internal constant FUNCTION_TREE_HEIGHT = 5;
  uint256 internal constant NOTE_HASH_TREE_HEIGHT = 32;
  uint256 internal constant PUBLIC_DATA_TREE_HEIGHT = 40;
  uint256 internal constant NULLIFIER_TREE_HEIGHT = 20;
  uint256 internal constant L1_TO_L2_MSG_TREE_HEIGHT = 16;
  uint256 internal constant ROLLUP_VK_TREE_HEIGHT = 8;
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
  uint256 internal constant FUNCTION_SELECTOR_NUM_BYTES = 4;
  uint256 internal constant ARGS_HASH_CHUNK_LENGTH = 64;
  uint256 internal constant ARGS_HASH_CHUNK_COUNT = 64;
  uint256 internal constant MAX_ARGS_LENGTH = 4096;
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
  uint256 internal constant CANONICAL_KEY_REGISTRY_ADDRESS =
    9735143693259978736521448915549382765209954358646272896519366195578572330622;
  uint256 internal constant DEPLOYER_CONTRACT_ADDRESS =
    1330791240588942273989478952163154931941860232471291360599950658792066893795;
  uint256 internal constant REGISTERER_CONTRACT_ADDRESS =
    12230492553436229472833564540666503591270810173190529382505862577652523721217;
  uint256 internal constant GAS_TOKEN_ADDRESS =
    21054354231481372816168706751151469079551620620213512837742215289221210616379;
  uint256 internal constant AZTEC_ADDRESS_LENGTH = 1;
  uint256 internal constant GAS_FEES_LENGTH = 2;
  uint256 internal constant GAS_LENGTH = 2;
  uint256 internal constant GAS_SETTINGS_LENGTH = 7;
  uint256 internal constant CALL_CONTEXT_LENGTH = 6;
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
  uint256 internal constant KEY_VALIDATION_REQUEST_LENGTH = 3;
  uint256 internal constant KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH = 4;
  uint256 internal constant SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH = 5;
  uint256 internal constant PARTIAL_STATE_REFERENCE_LENGTH = 6;
  uint256 internal constant READ_REQUEST_LENGTH = 2;
  uint256 internal constant LOG_HASH_LENGTH = 3;
  uint256 internal constant SCOPED_LOG_HASH_LENGTH = 4;
  uint256 internal constant ENCRYPTED_LOG_HASH_LENGTH = 4;
  uint256 internal constant SCOPED_ENCRYPTED_LOG_HASH_LENGTH = 5;
  uint256 internal constant NOTE_LOG_HASH_LENGTH = 4;
  uint256 internal constant NOTE_HASH_LENGTH = 2;
  uint256 internal constant SCOPED_NOTE_HASH_LENGTH = 4;
  uint256 internal constant NULLIFIER_LENGTH = 3;
  uint256 internal constant SCOPED_NULLIFIER_LENGTH = 4;
  uint256 internal constant CALLER_CONTEXT_LENGTH = 3;
  uint256 internal constant PRIVATE_CALL_REQUEST_LENGTH = 6;
  uint256 internal constant SCOPED_PRIVATE_CALL_REQUEST_LENGTH = 7;
  uint256 internal constant ROLLUP_VALIDATION_REQUESTS_LENGTH = 2;
  uint256 internal constant STATE_REFERENCE_LENGTH = 8;
  uint256 internal constant TX_CONTEXT_LENGTH = 9;
  uint256 internal constant TX_REQUEST_LENGTH = 13;
  uint256 internal constant TOTAL_FEES_LENGTH = 1;
  uint256 internal constant HEADER_LENGTH = 23;
  uint256 internal constant PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH = 457;
  uint256 internal constant PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH = 530;
  uint256 internal constant PRIVATE_CALL_STACK_ITEM_LENGTH = 460;
  uint256 internal constant PUBLIC_CONTEXT_INPUTS_LENGTH = 41;
  uint256 internal constant AGGREGATION_OBJECT_LENGTH = 16;
  uint256 internal constant SCOPED_READ_REQUEST_LEN = 3;
  uint256 internal constant PUBLIC_DATA_READ_LENGTH = 2;
  uint256 internal constant VALIDATION_REQUESTS_LENGTH = 1538;
  uint256 internal constant PUBLIC_DATA_UPDATE_REQUEST_LENGTH = 3;
  uint256 internal constant COMBINED_ACCUMULATED_DATA_LENGTH = 333;
  uint256 internal constant COMBINED_CONSTANT_DATA_LENGTH = 40;
  uint256 internal constant CALL_REQUEST_LENGTH = 7;
  uint256 internal constant PRIVATE_ACCUMULATED_DATA_LENGTH = 1152;
  uint256 internal constant PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH = 2739;
  uint256 internal constant PUBLIC_ACCUMULATED_DATA_LENGTH = 983;
  uint256 internal constant PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH = 3770;
  uint256 internal constant KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH = 383;
  uint256 internal constant CONSTANT_ROLLUP_DATA_LENGTH = 14;
  uint256 internal constant BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH = 31;
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
  uint256 internal constant RECURSIVE_PROOF_LENGTH = 93;
  uint256 internal constant NESTED_RECURSIVE_PROOF_LENGTH = 109;
  uint256 internal constant VERIFICATION_KEY_LENGTH_IN_FIELDS = 114;
}
