import { IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

import { LevelUp, LevelUpChain } from 'levelup';

import { IndexedTree } from '../interfaces/indexed_tree.js';
import { PreimageFactory } from '../standard_indexed_tree/standard_indexed_tree.js';
import { TreeBase } from '../tree_base.js';
import { BaseFullTreeSnapshot, BaseFullTreeSnapshotBuilder } from './base_full_snapshot.js';
import { IndexedTreeSnapshot, TreeSnapshotBuilder } from './snapshot_builder.js';

const snapshotLeafValue = (node: Buffer, index: bigint) =>
  Buffer.concat([Buffer.from('snapshot:leaf:'), node, Buffer.from(':' + index)]);

/** a */
export class IndexedTreeSnapshotBuilder
  extends BaseFullTreeSnapshotBuilder<IndexedTree & TreeBase, IndexedTreeSnapshot>
  implements TreeSnapshotBuilder<IndexedTreeSnapshot>
{
  constructor(db: LevelUp, tree: IndexedTree & TreeBase, private leafPreimageBuilder: PreimageFactory) {
    super(db, tree);
  }

  protected openSnapshot(root: Buffer, numLeaves: bigint): IndexedTreeSnapshot {
    return new IndexedTreeSnapshotImpl(this.db, root, numLeaves, this.tree, this.leafPreimageBuilder);
  }

  protected async handleLeaf(index: bigint, node: Buffer, batch: LevelUpChain) {
    const leafPreimage = await this.tree.getLatestLeafPreimageCopy(index, false);
    if (leafPreimage) {
      batch.put(snapshotLeafValue(node, index), leafPreimage.toBuffer());
    }
  }
}

/** A snapshot of an indexed tree at a particular point in time */
class IndexedTreeSnapshotImpl extends BaseFullTreeSnapshot implements IndexedTreeSnapshot {
  constructor(
    db: LevelUp,
    historicRoot: Buffer,
    numLeaves: bigint,
    tree: IndexedTree & TreeBase,
    private leafPreimageBuilder: PreimageFactory,
  ) {
    super(db, historicRoot, numLeaves, tree);
  }

  async getLeafValue(index: bigint): Promise<Buffer | undefined> {
    const leafPreimage = await this.getLatestLeafPreimageCopy(index);
    return leafPreimage?.toBuffer();
  }

  async getLatestLeafPreimageCopy(index: bigint): Promise<IndexedTreeLeafPreimage | undefined> {
    const leafNode = await super.getLeafValue(index);
    const leafValue = await this.db.get(snapshotLeafValue(leafNode!, index)).catch(() => undefined);
    if (leafValue) {
      return this.leafPreimageBuilder.fromBuffer(leafValue);
    } else {
      return undefined;
    }
  }

  async findIndexOfPreviousKey(newValue: bigint): Promise<{
    /**
     * The index of the found leaf.
     */
    index: bigint;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  }> {
    const numLeaves = this.getNumLeaves();
    const diff: bigint[] = [];

    for (let i = 0; i < numLeaves; i++) {
      // this is very inefficient
      const storedLeaf = await this.getLatestLeafPreimageCopy(BigInt(i))!;

      // The stored leaf can be undefined if it addresses an empty leaf
      // If the leaf is empty we do the same as if the leaf was larger
      if (storedLeaf === undefined) {
        diff.push(newValue);
      } else if (storedLeaf.getKey() > newValue) {
        diff.push(newValue);
      } else if (storedLeaf.getKey() === newValue) {
        return { index: BigInt(i), alreadyPresent: true };
      } else {
        diff.push(newValue - storedLeaf.getKey());
      }
    }

    let minIndex = 0;
    for (let i = 1; i < diff.length; i++) {
      if (diff[i] < diff[minIndex]) {
        minIndex = i;
      }
    }

    return { index: BigInt(minIndex), alreadyPresent: false };
  }

  async findLeafIndex(value: Buffer): Promise<bigint | undefined> {
    const index = await this.tree.findLeafIndex(value, false);
    if (index !== undefined && index < this.getNumLeaves()) {
      return index;
    }
  }
}
