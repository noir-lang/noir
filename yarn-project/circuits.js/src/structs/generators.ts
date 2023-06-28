/**
 * Enumerate the hash_indices which are used for pedersen hashing (copied from cpp).
 * @see circuits/cpp/src/aztec3/constants.hpp
 */
export enum GeneratorIndex {
  /**
   * Indices with size ≤ 8
   */
  COMMITMENT = 1,
  COMMITMENT_PLACEHOLDER,
  OUTER_COMMITMENT,
  NULLIFIER_HASHED_PRIVATE_KEY,
  NULLIFIER,
  INITIALISATION_NULLIFIER,
  OUTER_NULLIFIER,
  PUBLIC_DATA_READ,
  PUBLIC_DATA_UPDATE_REQUEST,
  FUNCTION_DATA,
  FUNCTION_LEAF,
  CONTRACT_DEPLOYMENT_DATA,
  CONSTRUCTOR,
  CONSTRUCTOR_ARGS,
  CONTRACT_ADDRESS,
  CONTRACT_LEAF,
  CALL_CONTEXT,
  CALL_STACK_ITEM,
  CALL_STACK_ITEM_2,
  L1_TO_L2_MESSAGE_SECRET,
  L2_TO_L1_MSG,
  TX_CONTEXT,
  PUBLIC_LEAF_INDEX,
  PUBLIC_DATA_LEAF,
  SIGNED_TX_REQUEST,
  GLOBAL_VARIABLES,
  PARTIAL_CONTRACT_ADDRESS,
  /**
   * Indices with size ≤ 16
   */
  TX_REQUEST = 33, // Size = 14
  /**
   * Indices with size ≤ 44
   */
  VK = 41, // Size = 35
  PRIVATE_CIRCUIT_PUBLIC_INPUTS, // Size = 39
  PUBLIC_CIRCUIT_PUBLIC_INPUTS, // Size = 32 (unused)
  FUNCTION_ARGS, // Size ≤ 40
}
