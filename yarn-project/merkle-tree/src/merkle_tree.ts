import { SiblingPath } from './sibling_path/sibling_path.js';

/**
 * Defines the possible Merkle tree IDs.
 */
export enum MerkleTreeId {
  CONTRACT_TREE = 0,
  CONTRACT_TREE_ROOTS_TREE = 1,
  NULLIFIER_TREE = 2,
}

/**
 * Defines the depths of the various merkle trees.
 */
export enum MerkleTreeDepths {
  CONTRACT_TREE = 32,
  CONTRACT_TREE_ROOTS_TREE = 8,
  NULLIFIER_TREE = 32,
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
 * Defines the interface for a source of sibling paths.
 */
export interface SiblingPathSource {
  getSiblingPath(index: bigint): Promise<SiblingPath>;
}

/**
 * Defines the interface for a Merkle tree.
 */
export interface MerkleTree extends SiblingPathSource {
  getRoot(): Buffer;
  getNumLeaves(): bigint;
  appendLeaves(leaves: Buffer[]): Promise<void>;
  commit(): Promise<void>;
  rollback(): Promise<void>;
}

/**
 * Defines the interface for a database that stores Merkle trees.
 */
export interface MerkleTreeDb {
  getTreeInfo(treeId: MerkleTreeId): Promise<TreeInfo>;
  appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void>;
  getSiblingPath(treeId: MerkleTreeId, index: bigint): Promise<SiblingPath>;
  commit(): Promise<void>;
  rollback(): Promise<void>;
}
