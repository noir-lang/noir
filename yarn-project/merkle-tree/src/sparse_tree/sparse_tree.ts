import { UpdateOnlyTree } from '../interfaces/update_only_tree.js';
import { FullTreeSnapshotBuilder } from '../snapshots/full_snapshot.js';
import { TreeSnapshot } from '../snapshots/snapshot_builder.js';
import { INITIAL_LEAF, TreeBase } from '../tree_base.js';

/**
 * A Merkle tree implementation that uses a LevelDB database to store the tree.
 */
export class SparseTree extends TreeBase implements UpdateOnlyTree {
  #snapshotBuilder = new FullTreeSnapshotBuilder(this.store, this);
  /**
   * Updates a leaf in the tree.
   * @param leaf - New contents of the leaf.
   * @param index - Index of the leaf to be updated.
   */
  public updateLeaf(leaf: Buffer, index: bigint): Promise<void> {
    if (index > this.maxIndex) {
      throw Error(`Index out of bounds. Index ${index}, max index: ${this.maxIndex}.`);
    }

    const insertingZeroElement = leaf.equals(INITIAL_LEAF);
    const originallyZeroElement = this.getLeafValue(index, true)?.equals(INITIAL_LEAF);
    if (insertingZeroElement && originallyZeroElement) {
      return Promise.resolve();
    }
    this.addLeafToCacheAndHashToRoot(leaf, index);
    if (insertingZeroElement) {
      // Deleting element (originally non-zero and new value is zero)
      this.cachedSize = (this.cachedSize ?? this.size) - 1n;
    } else if (originallyZeroElement) {
      // Inserting new element (originally zero and new value is non-zero)
      this.cachedSize = (this.cachedSize ?? this.size) + 1n;
    }

    return Promise.resolve();
  }

  public snapshot(block: number): Promise<TreeSnapshot> {
    return this.#snapshotBuilder.snapshot(block);
  }

  public getSnapshot(block: number): Promise<TreeSnapshot> {
    return this.#snapshotBuilder.getSnapshot(block);
  }

  public findLeafIndex(_value: Buffer, _includeUncommitted: boolean): bigint | undefined {
    throw new Error('Finding leaf index is not supported for sparse trees');
  }
}
