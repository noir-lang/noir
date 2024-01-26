import { IndexedTreeLeafPreimage } from '@aztec/foundation/trees';
import { AztecKVStore, AztecMap } from '@aztec/kv-store';

import { IndexedTree, PreimageFactory } from '../interfaces/indexed_tree.js';
import { TreeBase } from '../tree_base.js';
import { BaseFullTreeSnapshot, BaseFullTreeSnapshotBuilder } from './base_full_snapshot.js';
import { IndexedTreeSnapshot, TreeSnapshotBuilder } from './snapshot_builder.js';

const snapshotLeafValue = (node: Buffer, index: bigint) => 'snapshot:leaf:' + node.toString('hex') + ':' + index;

/** a */
export class IndexedTreeSnapshotBuilder
  extends BaseFullTreeSnapshotBuilder<IndexedTree & TreeBase, IndexedTreeSnapshot>
  implements TreeSnapshotBuilder<IndexedTreeSnapshot>
{
  leaves: AztecMap<string, Buffer>;
  constructor(store: AztecKVStore, tree: IndexedTree & TreeBase, private leafPreimageBuilder: PreimageFactory) {
    super(store, tree);
    this.leaves = store.openMap('indexed_tree_snapshot:' + tree.getName());
  }

  protected openSnapshot(root: Buffer, numLeaves: bigint): IndexedTreeSnapshot {
    return new IndexedTreeSnapshotImpl(this.nodes, this.leaves, root, numLeaves, this.tree, this.leafPreimageBuilder);
  }

  protected handleLeaf(index: bigint, node: Buffer) {
    const leafPreimage = this.tree.getLatestLeafPreimageCopy(index, false);
    if (leafPreimage) {
      void this.leaves.set(snapshotLeafValue(node, index), leafPreimage.toBuffer());
    }
  }
}

/** A snapshot of an indexed tree at a particular point in time */
class IndexedTreeSnapshotImpl extends BaseFullTreeSnapshot implements IndexedTreeSnapshot {
  constructor(
    db: AztecMap<string, [Buffer, Buffer]>,
    private leaves: AztecMap<string, Buffer>,
    historicRoot: Buffer,
    numLeaves: bigint,
    tree: IndexedTree & TreeBase,
    private leafPreimageBuilder: PreimageFactory,
  ) {
    super(db, historicRoot, numLeaves, tree);
  }

  getLeafValue(index: bigint): Buffer | undefined {
    const leafPreimage = this.getLatestLeafPreimageCopy(index);
    return leafPreimage?.toBuffer();
  }

  getLatestLeafPreimageCopy(index: bigint): IndexedTreeLeafPreimage | undefined {
    const leafNode = super.getLeafValue(index);
    const leafValue = this.leaves.get(snapshotLeafValue(leafNode!, index));
    if (leafValue) {
      return this.leafPreimageBuilder.fromBuffer(leafValue);
    } else {
      return undefined;
    }
  }

  findIndexOfPreviousKey(newValue: bigint): {
    /**
     * The index of the found leaf.
     */
    index: bigint;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  } {
    const numLeaves = this.getNumLeaves();
    const diff: bigint[] = [];

    for (let i = 0; i < numLeaves; i++) {
      // this is very inefficient
      const storedLeaf = this.getLatestLeafPreimageCopy(BigInt(i))!;

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

  findLeafIndex(value: Buffer): bigint | undefined {
    const index = this.tree.findLeafIndex(value, false);
    if (index !== undefined && index < this.getNumLeaves()) {
      return index;
    }
  }
}
