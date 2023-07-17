import { SiblingPath } from '@aztec/types';

import { LowLeafWitnessData } from '../index.js';
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

/**
 * Indexed merkle tree.
 */
export interface IndexedTree extends AppendOnlyTree {
  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newValue - The new value to be inserted into the tree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns The found leaf index and a flag indicating if the corresponding leaf's value is equal to `newValue`.
   */
  findIndexOfPreviousValue(
    newValue: bigint,
    includeUncommitted: boolean,
  ): {
    /**
     * The index of the found leaf.
     */
    index: number;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  };

  /**
   * Gets the latest LeafData copy.
   * @param index - Index of the leaf of which to obtain the LeafData copy.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns A copy of the leaf data at the given index or undefined if the leaf was not found.
   */
  getLatestLeafDataCopy(index: number, includeUncommitted: boolean): LeafData | undefined;

  /**
   * Batch insert multiple leaves into the tree.
   * @param leaves - Leaves to insert into the tree.
   * @param subtreeHeight - Height of the subtree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   */
  batchInsert<TreeHeight extends number, SubtreeHeight extends number, SubtreeSiblingPathHeight extends number>(
    leaves: Buffer[],
    subtreeHeight: SubtreeHeight,
    includeUncommitted: boolean,
  ): Promise<
    | [LowLeafWitnessData<TreeHeight>[], SiblingPath<SubtreeSiblingPathHeight>]
    | [undefined, SiblingPath<SubtreeSiblingPathHeight>]
  >;
}
