import { LeafData, SiblingPath } from '@aztec/types';

/**
 * An interface for a tree that can record snapshots of its contents.
 */
export interface TreeSnapshotBuilder<S extends TreeSnapshot = TreeSnapshot> {
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
export interface TreeSnapshot {
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
  getLeafValue(index: bigint): Promise<Buffer | undefined>;

  /**
   * Returns the sibling path for a requested leaf index.
   * @param index - The index of the leaf for which a sibling path is required.
   */
  getSiblingPath<N extends number>(index: bigint): Promise<SiblingPath<N>>;
}

/** A snapshot of an indexed tree */
export interface IndexedTreeSnapshot extends TreeSnapshot {
  /**
   * Gets the historical data for a leaf
   * @param index - The index of the leaf to get the data for
   */
  getLatestLeafDataCopy(index: bigint): Promise<LeafData | undefined>;

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newValue - The new value to be inserted into the tree.
   * @returns The found leaf index and a flag indicating if the corresponding leaf's value is equal to `newValue`.
   */
  findIndexOfPreviousValue(newValue: bigint): Promise<{
    /**
     * The index of the found leaf.
     */
    index: number;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  }>;
}
