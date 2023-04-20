import { AppendOnlyTree } from './append_only_tree.js';

/**
 * A leaf of a tree.
 */
export interface LeafData {
  /**
   * A value of the leaf.
   */
  value: bigint;
  /**
   * An index of the next leaf.
   */
  nextIndex: bigint;
  /**
   * A value of the next leaf.
   */
  nextValue: bigint;
}

export interface IndexedTree extends AppendOnlyTree {
  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newValue - The new value to be inserted into the tree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns Tuple containing the leaf index and a flag to say if the value is a duplicate.
   */
  findIndexOfPreviousValue(newValue: bigint, includeUncommitted: boolean): { index: number; alreadyPresent: boolean };

  /**
   * Gets the latest LeafData copy.
   * @param index - Index of the leaf of which to obtain the LeafData copy.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns A copy of the leaf data at the given index or undefined if the leaf was not found.
   */
  getLatestLeafDataCopy(index: number, includeUncommitted: boolean): LeafData | undefined;

  /**
   * Exposes the underlying tree's update leaf method
   * @param leaf - The hash to set at the leaf
   * @param index - The index of the element
   */
  // TODO: remove once the batch insertion functionality is moved to StandardIndexedTree from circuit_block_builder.ts
  updateLeaf(leaf: LeafData, index: bigint): Promise<void>;
}
