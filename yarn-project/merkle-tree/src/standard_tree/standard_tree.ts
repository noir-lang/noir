import { TreeInsertionStats } from '@aztec/circuit-types/stats';
import { Timer } from '@aztec/foundation/timer';

import { AppendOnlyTree } from '../interfaces/append_only_tree.js';
import { AppendOnlySnapshotBuilder } from '../snapshots/append_only_snapshot.js';
import { TreeSnapshot } from '../snapshots/snapshot_builder.js';
import { TreeBase } from '../tree_base.js';

/**
 * A Merkle tree implementation that uses a LevelDB database to store the tree.
 */
export class StandardTree extends TreeBase implements AppendOnlyTree {
  #snapshotBuilder = new AppendOnlySnapshotBuilder(this.store, this, this.hasher);

  /**
   * Appends the given leaves to the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   */
  public appendLeaves(leaves: Buffer[]): Promise<void> {
    this.hasher.reset();
    const timer = new Timer();
    super.appendLeaves(leaves);
    this.log(`Inserted ${leaves.length} leaves into ${this.getName()} tree`, {
      eventName: 'tree-insertion',
      duration: timer.ms(),
      batchSize: leaves.length,
      treeName: this.getName(),
      treeDepth: this.getDepth(),
      treeType: 'append-only',
      ...this.hasher.stats(),
    } satisfies TreeInsertionStats);

    return Promise.resolve();
  }

  public snapshot(blockNumber: number): Promise<TreeSnapshot> {
    return this.#snapshotBuilder.snapshot(blockNumber);
  }

  public getSnapshot(blockNumber: number): Promise<TreeSnapshot> {
    return this.#snapshotBuilder.getSnapshot(blockNumber);
  }

  public findLeafIndex(value: Buffer, includeUncommitted: boolean): bigint | undefined {
    for (let i = 0n; i < this.getNumLeaves(includeUncommitted); i++) {
      const currentValue = this.getLeafValue(i, includeUncommitted);
      if (currentValue && currentValue.equals(value)) {
        return i;
      }
    }
    return undefined;
  }
}
