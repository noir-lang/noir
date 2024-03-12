/**
 * A leaf of an indexed merkle tree.
 */
export interface IndexedTreeLeaf {
  /**
   * Returns key of the leaf. It's used for indexing.
   */
  getKey(): bigint;
  /**
   * Serializes the leaf into a buffer.
   */
  toBuffer(): Buffer;
  /**
   * Returns true if the leaf is empty.
   */
  isEmpty(): boolean;
  /**
   * Updates the leaf with the data of another leaf.
   * @param another - The leaf to update to.
   * @returns The updated leaf.
   */
  updateTo(another: IndexedTreeLeaf): IndexedTreeLeaf;
}

/**
 * Preimage of a merkle tree leaf.
 */
export interface TreeLeafPreimage {
  /**
   * Returns key of the leaf corresponding to this preimage.
   */
  getKey(): bigint;
  /**
   * Returns the preimage as a leaf.
   */
  asLeaf(): IndexedTreeLeaf;
  /**
   * Serializes the preimage into a buffer.
   */
  toBuffer(): Buffer;
  /**
   * Serializes the preimage to an array of buffers for hashing.
   */
  toHashInputs(): Buffer[];
}

/**
 * Preimage of an indexed merkle tree leaf.
 */
export interface IndexedTreeLeafPreimage extends TreeLeafPreimage {
  getNextKey(): bigint;
  /**
   * Returns the index of the next leaf.
   */
  getNextIndex(): bigint;
}
