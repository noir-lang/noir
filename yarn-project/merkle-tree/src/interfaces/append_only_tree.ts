import { Bufferable } from '@aztec/foundation/serialize';

import { TreeSnapshot, TreeSnapshotBuilder } from '../snapshots/snapshot_builder.js';
import { MerkleTree } from './merkle_tree.js';

/**
 * A Merkle tree that supports only appending leaves and not updating existing leaves.
 */
export interface AppendOnlyTree<T extends Bufferable = Buffer>
  extends MerkleTree<T>,
    TreeSnapshotBuilder<TreeSnapshot<T>> {
  /**
   * Appends a set of leaf values to the tree.
   * @param leaves - The set of leaves to be appended.
   */
  appendLeaves(leaves: T[]): Promise<void>;
}
