import { AppendOnlySnapshotBuilder, TreeSnapshot } from '../index.js';
import { AppendOnlyTree } from '../interfaces/append_only_tree.js';
import { TreeBase } from '../tree_base.js';

/**
 * A Merkle tree implementation that uses a LevelDB database to store the tree.
 */
export class StandardTree extends TreeBase implements AppendOnlyTree {
  #snapshotBuilder = new AppendOnlySnapshotBuilder(this.db, this, this.hasher);

  /**
   * Appends the given leaves to the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   */
  public async appendLeaves(leaves: Buffer[]): Promise<void> {
    await super.appendLeaves(leaves);
  }

  public snapshot(block: number): Promise<TreeSnapshot> {
    return this.#snapshotBuilder.snapshot(block);
  }

  public getSnapshot(block: number): Promise<TreeSnapshot> {
    return this.#snapshotBuilder.getSnapshot(block);
  }
}
