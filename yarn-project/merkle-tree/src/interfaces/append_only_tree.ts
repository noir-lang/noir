import { MerkleTree } from './merkle_tree.js';

export interface AppendOnlyTree extends MerkleTree {
  /**
   * Appends a set of leaf values to the tree
   * @param leaves - The set of leaves to be appended
   */
  appendLeaves(leaves: Buffer[]): Promise<void>;
}
