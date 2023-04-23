import { MerkleTree } from './merkle_tree.js';

/**
 * A Merkle tree that supports only appending leaves and not updating existing leaves.
 */
export interface AppendOnlyTree extends MerkleTree {
  /**
   * Appends a set of leaf values to the tree.
   * @param leaves - The set of leaves to be appended.
   */
  appendLeaves(leaves: Buffer[]): Promise<void>;
}
