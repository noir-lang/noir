// See aztec3/constants.hpp
// Copied here for prototyping purposes
// In future: structured serialization?
export const ARGS_LENGTH = 8;
export const RETURN_VALUES_LENGTH = 4;
export const EMITTED_EVENTS_LENGTH = 4;

export const NEW_COMMITMENTS_LENGTH = 4;
export const NEW_NULLIFIERS_LENGTH = 4;

export const STATE_TRANSITIONS_LENGTH = 4;
export const STATE_READS_LENGTH = 4;

export const PRIVATE_CALL_STACK_LENGTH = 4;
export const PUBLIC_CALL_STACK_LENGTH = 4;
export const L1_MSG_STACK_LENGTH = 2;

export const KERNEL_NEW_COMMITMENTS_LENGTH = 4;
export const KERNEL_NEW_NULLIFIERS_LENGTH = 4;
export const KERNEL_NEW_CONTRACTS_LENGTH = 1;
export const KERNEL_PRIVATE_CALL_STACK_LENGTH = 8;
export const KERNEL_PUBLIC_CALL_STACK_LENGTH = 8;
export const KERNEL_L1_MSG_STACK_LENGTH = 4;
export const KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH = 4;

export const VK_TREE_HEIGHT = 3;
export const FUNCTION_TREE_HEIGHT = 4;
export const CONTRACT_TREE_HEIGHT = 4;
export const PRIVATE_DATA_TREE_HEIGHT = 8;
export const PUBLIC_DATA_TREE_HEIGHT = 32;
export const NULLIFIER_TREE_HEIGHT = 8;

export const PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT = 8;
export const CONTRACT_TREE_ROOTS_TREE_HEIGHT = 8;
export const ROLLUP_VK_TREE_HEIGHT = 8;

export const FUNCTION_SELECTOR_NUM_BYTES = 31;
