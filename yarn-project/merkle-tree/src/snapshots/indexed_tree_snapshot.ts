import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { LeafData } from '@aztec/types';

import { LevelUp, LevelUpChain } from 'levelup';

import { IndexedTree } from '../interfaces/indexed_tree.js';
import { decodeTreeValue, encodeTreeValue } from '../standard_indexed_tree/standard_indexed_tree.js';
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
  constructor(db: LevelUp, tree: IndexedTree & TreeBase) {
    super(db, tree);
  }

  protected openSnapshot(root: Buffer, numLeaves: bigint): IndexedTreeSnapshot {
    return new IndexedTreeSnapshotImpl(this.db, root, numLeaves, this.tree);
  }

  protected handleLeaf(index: bigint, node: Buffer, batch: LevelUpChain) {
    const leafData = this.tree.getLatestLeafDataCopy(Number(index), false);
    if (leafData) {
      batch.put(snapshotLeafValue(node, index), encodeTreeValue(leafData));
    }
  }
}

/** A snapshot of an indexed tree at a particular point in time */
class IndexedTreeSnapshotImpl extends BaseFullTreeSnapshot implements IndexedTreeSnapshot {
  async getLeafValue(index: bigint): Promise<Buffer | undefined> {
    const leafData = await this.getLatestLeafDataCopy(index);
    return leafData ? toBufferBE(leafData.value, 32) : undefined;
  }

  async getLatestLeafDataCopy(index: bigint): Promise<LeafData | undefined> {
    const leafNode = await super.getLeafValue(index);
    const leafValue = await this.db.get(snapshotLeafValue(leafNode!, index)).catch(() => undefined);
    if (leafValue) {
      return decodeTreeValue(leafValue);
    } else {
      return undefined;
    }
  }

  async findIndexOfPreviousValue(newValue: bigint): Promise<{
    /**
     * The index of the found leaf.
     */
    index: number;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  }> {
    const numLeaves = this.getNumLeaves();
    const diff: bigint[] = [];

    for (let i = 0; i < numLeaves; i++) {
      // this is very inefficient
      const storedLeaf = await this.getLatestLeafDataCopy(BigInt(i))!;

      // The stored leaf can be undefined if it addresses an empty leaf
      // If the leaf is empty we do the same as if the leaf was larger
      if (storedLeaf === undefined) {
        diff.push(newValue);
      } else if (storedLeaf.value > newValue) {
        diff.push(newValue);
      } else if (storedLeaf.value === newValue) {
        return { index: i, alreadyPresent: true };
      } else {
        diff.push(newValue - storedLeaf.value);
      }
    }

    let minIndex = 0;
    for (let i = 1; i < diff.length; i++) {
      if (diff[i] < diff[minIndex]) {
        minIndex = i;
      }
    }

    return { index: minIndex, alreadyPresent: false };
  }
}
