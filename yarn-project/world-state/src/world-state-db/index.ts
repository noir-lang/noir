import { SiblingPath } from '@aztec/merkle-tree';

export * from './memory_world_state_db.js';

/**
 * Defines the possible Merkle tree IDs.
 */
export enum MerkleTreeId {
  CONTRACT_TREE = 0,
  CONTRACT_TREE_ROOTS_TREE = 1,
  NULLIFIER_TREE = 2,
  DATA_TREE = 3,
  DATA_TREE_ROOTS_TREE = 4,
}

/**
 *  Defines tree information.
 */
export interface TreeInfo {
  /**
   * The tree ID.
   */
  treeId: MerkleTreeId;
  /**
   * The tree root.
   */
  root: Buffer;
  /**
   * The number of leaves in the tree.
   */
  size: bigint;
}

/**
 * Defines the interface for Merkle Tree operations.
 */
export interface MerkleTreeOperations {
  getTreeInfo(treeId: MerkleTreeId): Promise<TreeInfo>;
  appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void>;
  getSiblingPath(treeId: MerkleTreeId, index: bigint): Promise<SiblingPath>;
}

/**
 * Defines the interface for a database that stores Merkle trees.
 */
export interface MerkleTreeDb extends MerkleTreeOperations {
  commit(): Promise<void>;
  rollback(): Promise<void>;
}
