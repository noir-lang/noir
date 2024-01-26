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
  uint256 internal constant RETURN_VALUES_LENGTH = 4;
  uint256 internal constant MAX_NEW_COMMITMENTS_PER_CALL = 16;
  uint256 internal constant MAX_NEW_NULLIFIERS_PER_CALL = 16;
  uint256 internal constant MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL = 4;
  uint256 internal constant MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL = 4;
  uint256 internal constant MAX_NEW_L2_TO_L1_MSGS_PER_CALL = 2;
  uint256 internal constant MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL = 16;
  uint256 internal constant MAX_PUBLIC_DATA_READS_PER_CALL = 16;
  uint256 internal constant MAX_READ_REQUESTS_PER_CALL = 32;
  uint256 internal constant MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL = 1;
  uint256 internal constant MAX_NEW_COMMITMENTS_PER_TX = 64;
  uint256 internal constant MAX_NEW_NULLIFIERS_PER_TX = 64;
  uint256 internal constant MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX = 8;
  uint256 internal constant MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX = 8;
  uint256 internal constant MAX_NEW_L2_TO_L1_MSGS_PER_TX = 2;
  uint256 internal constant MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX = 16;
  uint256 internal constant MAX_PUBLIC_DATA_READS_PER_TX = 16;
  uint256 internal constant MAX_NEW_CONTRACTS_PER_TX = 1;
  uint256 internal constant MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX = 4;
  uint256 internal constant MAX_READ_REQUESTS_PER_TX = 128;
  uint256 internal constant MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX = 4;
  uint256 internal constant NUM_ENCRYPTED_LOGS_HASHES_PER_TX = 1;
  uint256 internal constant NUM_UNENCRYPTED_LOGS_HASHES_PER_TX = 1;
  uint256 internal constant NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP = 16;
  uint256 internal constant VK_TREE_HEIGHT = 3;
  uint256 internal constant FUNCTION_TREE_HEIGHT = 5;
  uint256 internal constant CONTRACT_TREE_HEIGHT = 16;
  uint256 internal constant NOTE_HASH_TREE_HEIGHT = 32;
  uint256 internal constant PUBLIC_DATA_TREE_HEIGHT = 40;
  uint256 internal constant NULLIFIER_TREE_HEIGHT = 20;
  uint256 internal constant L1_TO_L2_MSG_TREE_HEIGHT = 16;
  uint256 internal constant ROLLUP_VK_TREE_HEIGHT = 8;
  uint256 internal constant CONTRACT_SUBTREE_HEIGHT = 0;
  uint256 internal constant CONTRACT_SUBTREE_SIBLING_PATH_LENGTH = 16;
  uint256 internal constant NOTE_HASH_SUBTREE_HEIGHT = 6;
  uint256 internal constant NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH = 26;
  uint256 internal constant NULLIFIER_SUBTREE_HEIGHT = 6;
  uint256 internal constant PUBLIC_DATA_SUBTREE_HEIGHT = 4;
  uint256 internal constant ARCHIVE_HEIGHT = 16;
  uint256 internal constant NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH = 14;
  uint256 internal constant PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH = 36;
  uint256 internal constant L1_TO_L2_MSG_SUBTREE_HEIGHT = 4;
  uint256 internal constant L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH = 12;
  uint256 internal constant FUNCTION_SELECTOR_NUM_BYTES = 4;
  uint256 internal constant MAPPING_SLOT_PEDERSEN_SEPARATOR = 4;
  uint256 internal constant NUM_FIELDS_PER_SHA256 = 2;
  uint256 internal constant ARGS_HASH_CHUNK_LENGTH = 32;
  uint256 internal constant ARGS_HASH_CHUNK_COUNT = 16;
  uint256 internal constant L1_TO_L2_MESSAGE_LENGTH = 8;
  uint256 internal constant L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH = 25;
  uint256 internal constant MAX_NOTE_FIELDS_LENGTH = 20;
  uint256 internal constant GET_NOTE_ORACLE_RETURN_LENGTH = 23;
  uint256 internal constant MAX_NOTES_PER_PAGE = 10;
  uint256 internal constant VIEW_NOTE_ORACLE_RETURN_LENGTH = 212;
  uint256 internal constant CALL_CONTEXT_LENGTH = 8;
  uint256 internal constant BLOCK_HEADER_LENGTH = 7;
  uint256 internal constant FUNCTION_DATA_LENGTH = 4;
  uint256 internal constant CONTRACT_DEPLOYMENT_DATA_LENGTH = 6;
  uint256 internal constant PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH = 189;
  uint256 internal constant CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH = 3;
  uint256 internal constant CONTRACT_STORAGE_READ_LENGTH = 2;
  uint256 internal constant PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH = 190;
  uint256 internal constant GET_NOTES_ORACLE_RETURN_LENGTH = 674;
  uint256 internal constant CALL_PRIVATE_FUNCTION_RETURN_SIZE = 199;
  uint256 internal constant PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH = 87;
  uint256 internal constant PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH = 177;
  uint256 internal constant COMMITMENTS_NUM_BYTES_PER_BASE_ROLLUP = 2048;
  uint256 internal constant NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP = 2048;
  uint256 internal constant PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP = 1024;
  uint256 internal constant CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP = 32;
  uint256 internal constant CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP = 64;
  uint256 internal constant CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED = 52;
  uint256 internal constant L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP = 64;
  uint256 internal constant LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP = 64;
}
