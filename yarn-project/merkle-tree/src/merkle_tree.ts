import { LeafData } from './index.js';
import { SiblingPath } from './sibling_path/sibling_path.js';

/**
 * Defines the interface for a source of sibling paths.
 */
export interface SiblingPathSource {
  /**
   * Returns the sibling path for a requested leaf index
   * @param index - The index of the leaf for which a sibling path is required
   * @param includeUncommitted - Set to true to include uncommitted updates in the sibling path
   */
  getSiblingPath(index: bigint, includeUncommitted: boolean): Promise<SiblingPath>;
}

/**
 * Defines the interface for a Merkle tree.
 */
export interface MerkleTree extends SiblingPathSource {
  /**
   * Returns the current root of the tree
   * @param includeUncommitted - Set to true to include uncommitted updates in the calculated root
   */
  getRoot(includeUncommitted: boolean): Buffer;
  /**
   * Returns the number of leaves in the tree
   * @param includeUncommitted - Set to true to include uncommitted updates in the returned value
   */
  getNumLeaves(includeUncommitted: boolean): bigint;
  /**
   * Appends a set of leaf values to the tree
   * @param leaves - The set of leaves to be appended
   */
  appendLeaves(leaves: Buffer[]): Promise<void>;
  /**
   * Commit pending updates to the tree
   */
  commit(): Promise<void>;
  /**
   * Updates a leaf at a given index in the tree
   * @param leaf The leaf value to be updated
   * @param index The leaf to be updated
   */
  updateLeaf(leaf: Buffer | LeafData, index: bigint): Promise<void>;
  /**
   * Returns the depth of the tree
   */
  getDepth(): number;
  /**
   * Rollback pending update to the tree
   */
  rollback(): Promise<void>;
  /**
   * Returns the value of a leaf at the specified index
   * @param index - The index of the leaf value to be returned
   * @param includeUncommitted - Set to true to include uncommitted updates in the data set
   */
  getLeafValue(index: bigint, includeUncommitted: boolean): Promise<Buffer | undefined>;
}
