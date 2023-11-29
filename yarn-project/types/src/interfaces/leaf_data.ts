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
