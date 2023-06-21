// See aztec3/constants.hpp
// Copied here for prototyping purposes
// In future: structured serialization?
export const ARGS_LENGTH = 16; // MAX_ARGS in Noir
export const RETURN_VALUES_LENGTH = 4;

export const READ_REQUESTS_LENGTH = 4;

export const NEW_COMMITMENTS_LENGTH = 4;
export const NEW_NULLIFIERS_LENGTH = 4;
export const NEW_L2_TO_L1_MSGS_LENGTH = 2;

export const PRIVATE_CALL_STACK_LENGTH = 4;
export const PUBLIC_CALL_STACK_LENGTH = 4;

export const KERNEL_NEW_COMMITMENTS_LENGTH = 4;
export const KERNEL_NEW_NULLIFIERS_LENGTH = 4;
export const KERNEL_NEW_CONTRACTS_LENGTH = 1;
export const KERNEL_PRIVATE_CALL_STACK_LENGTH = 8;
export const KERNEL_PUBLIC_CALL_STACK_LENGTH = 8;
export const KERNEL_NEW_L2_TO_L1_MSGS_LENGTH = 2;
export const KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH = 4;
export const KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH = 4;
export const KERNEL_PUBLIC_DATA_READS_LENGTH = 4;

export const VK_TREE_HEIGHT = 3;
export const FUNCTION_TREE_HEIGHT = 4;
export const CONTRACT_TREE_HEIGHT = 8;
export const PRIVATE_DATA_TREE_HEIGHT = 8;
export const PUBLIC_DATA_TREE_HEIGHT = 254;
export const NULLIFIER_TREE_HEIGHT = 8;
export const NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP = 16;
export const L1_TO_L2_MESSAGES_TREE_HEIGHT = 8;
export const L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT = 8;
export const L1_TO_L2_MESSAGES_SUBTREE_HEIGHT = Math.ceil(Math.log2(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP));
export const L1_TO_L2_MESSAGES_SIBLING_PATH_LENGTH = L1_TO_L2_MESSAGES_TREE_HEIGHT - L1_TO_L2_MESSAGES_SUBTREE_HEIGHT;

export const PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT = 8;
export const CONTRACT_TREE_ROOTS_TREE_HEIGHT = 8;
export const ROLLUP_VK_TREE_HEIGHT = 8;

export const FUNCTION_SELECTOR_NUM_BYTES = 4;

// sha256 hash is stored in two fields to accommodate all 256-bits of the hash
export const NUM_FIELDS_PER_SHA256 = 2;
