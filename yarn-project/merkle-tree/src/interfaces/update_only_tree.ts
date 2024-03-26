import { Bufferable } from '@aztec/foundation/serialize';

import { TreeSnapshot, TreeSnapshotBuilder } from '../snapshots/snapshot_builder.js';
import { MerkleTree } from './merkle_tree.js';

/**
 * A Merkle tree that supports updates at arbitrary indices but not appending.
 */
export interface UpdateOnlyTree<T extends Bufferable = Buffer>
  extends MerkleTree<T>,
    TreeSnapshotBuilder<TreeSnapshot<T>> {
  /**
   * Updates a leaf at a given index in the tree.
   * @param leaf - The leaf value to be updated.
   * @param index - The leaf to be updated.
   */
  updateLeaf(leaf: T, index: bigint): Promise<void>;
}
