import { type SiblingPath } from '@aztec/circuit-types';
import { type Bufferable } from '@aztec/foundation/serialize';
import { type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

/**
 * An interface for a tree that can record snapshots of its contents.
 */
export interface TreeSnapshotBuilder<S extends TreeSnapshot<Bufferable>> {
  /**
   * Creates a snapshot of the tree at the given version.
   * @param block - The version to snapshot the tree at.
   */
  snapshot(block: number): Promise<S>;

  /**
   * Returns a snapshot of the tree at the given version.
   * @param block - The version of the snapshot to return.
   */
  getSnapshot(block: number): Promise<S>;
}

/**
 * A tree snapshot
 */
export interface TreeSnapshot<T extends Bufferable> {
  /**
   * Returns the current root of the tree.
   */
  getRoot(): Buffer;

  /**
   * Returns the number of leaves in the tree.
   */
  getDepth(): number;

  /**
   * Returns the number of leaves in the tree.
   */
  getNumLeaves(): bigint;

  /**
   * Returns the value of a leaf at the specified index.
   * @param index - The index of the leaf value to be returned.
   */
  getLeafValue(index: bigint): T | undefined;

  /**
   * Returns the sibling path for a requested leaf index.
   * @param index - The index of the leaf for which a sibling path is required.
   */
  getSiblingPath<N extends number>(index: bigint): SiblingPath<N>;

  /**
   * Returns the index of a leaf given its value, or undefined if no leaf with that value is found.
   * @param treeId - The ID of the tree.
   * @param value - The leaf value to look for.
   * @returns The index of the first leaf found with a given value (undefined if not found).
   */
  findLeafIndex(value: T): bigint | undefined;

  /**
   * Returns the first index containing a leaf value after `startIndex`.
   * @param leaf - The leaf value to look for.
   * @param startIndex - The index to start searching from (used when skipping nullified messages)
   * @returns The index of the first leaf found with a given value (undefined if not found).
   */
  findLeafIndexAfter(leaf: T, startIndex: bigint): bigint | undefined;
}

/** A snapshot of an indexed tree */
export interface IndexedTreeSnapshot extends TreeSnapshot<Buffer> {
  /**
   * Gets the historical data for a leaf
   * @param index - The index of the leaf to get the data for
   */
  getLatestLeafPreimageCopy(index: bigint): IndexedTreeLeafPreimage | undefined;

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newValue - The new value to be inserted into the tree.
   * @returns The found leaf index and a flag indicating if the corresponding leaf's value is equal to `newValue`.
   */
  findIndexOfPreviousKey(newValue: bigint): {
    /**
     * The index of the found leaf.
     */
    index: bigint;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  };
}
