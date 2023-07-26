/**
 * Defines the possible Merkle tree IDs.
 */
export enum MerkleTreeId {
  CONTRACT_TREE = 0,
  CONTRACT_TREE_ROOTS_TREE = 1,
  NULLIFIER_TREE = 2,
  PRIVATE_DATA_TREE = 3,
  PRIVATE_DATA_TREE_ROOTS_TREE = 4,
  PUBLIC_DATA_TREE = 5,
  L1_TO_L2_MESSAGES_TREE = 6,
  L1_TO_L2_MESSAGES_ROOTS_TREE = 7,
  BLOCKS_TREE = 8,
}

export const merkleTreeIds = () => {
  return Object.values(MerkleTreeId).filter((v): v is MerkleTreeId => !isNaN(Number(v)));
};
