import {
  ARCHIVE_TREE_ID,
  L1_TO_L2_MESSAGE_TREE_ID,
  NOTE_HASH_TREE_ID,
  NULLIFIER_TREE_ID,
  PUBLIC_DATA_TREE_ID,
} from '@aztec/circuits.js';

/**
 * Defines the possible Merkle tree IDs.
 * @remarks The MerkleTrees class expects these to start from zero and be in incremental order.
 */
export enum MerkleTreeId {
  NULLIFIER_TREE = NULLIFIER_TREE_ID,
  NOTE_HASH_TREE = NOTE_HASH_TREE_ID,
  PUBLIC_DATA_TREE = PUBLIC_DATA_TREE_ID,
  L1_TO_L2_MESSAGE_TREE = L1_TO_L2_MESSAGE_TREE_ID,
  ARCHIVE = ARCHIVE_TREE_ID,
}

export const merkleTreeIds = () => {
  return Object.values(MerkleTreeId).filter((v): v is MerkleTreeId => !isNaN(Number(v)));
};
