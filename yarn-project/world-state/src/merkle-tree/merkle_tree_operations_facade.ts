import { SiblingPath } from '@aztec/merkle-tree';
import { LeafData, MerkleTreeDbOperations, MerkleTreeId, MerkleTreeOperations, TreeInfo } from '../index.js';

/**
 * Wraps a MerkleTreeDbOperations to call all functions with a preset includeUncommitted flag.
 */
export class MerkleTreeOperationsFacade implements MerkleTreeOperations {
  constructor(private trees: MerkleTreeDbOperations, private includeUncommitted: boolean) {}

  /**
   * Returns the tree info for the specified tree id.
   * @param treeId - Id of the tree to get information from.
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   * @returns The tree info for the specified tree.
   */
  getTreeInfo(treeId: MerkleTreeId): Promise<TreeInfo> {
    return this.trees.getTreeInfo(treeId, this.includeUncommitted);
  }

  /**
   * Appends a set of leaf values to the tree.
   * @param treeId - Id of the tree to append leaves to.
   * @param leaves - The set of leaves to be appended.
   * @returns The tree info of the specified tree.
   */
  appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void> {
    return this.trees.appendLeaves(treeId, leaves);
  }

  /**
   * Returns the sibling path for a requested leaf index.
   * @param treeId - Id of the tree to get the sibling path from.
   * @param index - The index of the leaf for which a sibling path is required.
   * @returns A promise with the sibling path of the specified leaf index.
   */
  getSiblingPath(treeId: MerkleTreeId, index: bigint): Promise<SiblingPath> {
    return this.trees.getSiblingPath(treeId, index, this.includeUncommitted);
  }

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param treeId - The ID of the tree to search.
   * @param value - The value to be inserted into the tree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns The found leaf index and a flag indicating if the corresponding leaf's value is equal to `newValue`.
   */
  getPreviousValueIndex(
    treeId: MerkleTreeId.NULLIFIER_TREE,
    value: bigint,
  ): Promise<{
    /**
     * The index of the found leaf.
     */
    index: number;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  }> {
    return this.trees.getPreviousValueIndex(treeId, value, this.includeUncommitted);
  }

  /**
   * Updates a leaf in a tree at a given index.
   * @param treeId - The ID of the tree.
   * @param leaf - The new leaf value.
   * @param index - The index to insert into.
   * @returns Empty promise.
   */
  updateLeaf(treeId: MerkleTreeId.NULLIFIER_TREE, leaf: LeafData, index: bigint): Promise<void> {
    return this.trees.updateLeaf(treeId, leaf, index, this.includeUncommitted);
  }

  /**
   * Gets the leaf data at a given index and tree.
   * @param treeId - The ID of the tree get the leaf from.
   * @param index - The index of the leaf to get.
   * @returns Leaf data.
   */
  getLeafData(treeId: MerkleTreeId.NULLIFIER_TREE, index: number): Promise<LeafData | undefined> {
    return this.trees.getLeafData(treeId, index, this.includeUncommitted);
  }

  /**
   * Returns the index of a leaf given its value, or undefined if no leaf with that value is found.
   * @param treeId - The ID of the tree.
   * @param value - The leaf value to look for.
   * @returns The index of the first leaf found with a given value (undefined if not found).
   */
  findLeafIndex(treeId: MerkleTreeId, value: Buffer): Promise<bigint | undefined> {
    return this.trees.findLeafIndex(treeId, value, this.includeUncommitted);
  }

  /**
   * Gets the value at the given index.
   * @param treeId - The ID of the tree to get the leaf value from.
   * @param index - The index of the leaf.
   * @param includeUncommitted - Indicates whether to include uncommitted changes.
   * @returns Leaf value at the given index (undefined if not found).
   */
  getLeafValue(treeId: MerkleTreeId, index: bigint): Promise<Buffer | undefined> {
    return this.trees.getLeafValue(treeId, index, this.includeUncommitted);
  }
}
