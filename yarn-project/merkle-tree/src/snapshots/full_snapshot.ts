import { type Bufferable, type FromBuffer } from '@aztec/foundation/serialize';
import { type AztecKVStore } from '@aztec/kv-store';

import { type TreeBase } from '../tree_base.js';
import { BaseFullTreeSnapshot, BaseFullTreeSnapshotBuilder } from './base_full_snapshot.js';
import { type TreeSnapshot, type TreeSnapshotBuilder } from './snapshot_builder.js';

/**
 * Builds a full snapshot of a tree. This implementation works for any Merkle tree and stores
 * it in a database in a similar way to how a tree is stored in memory, using pointers.
 *
 * Sharing the same database between versions and trees is recommended as the trees would share
 * structure.
 *
 * Complexity:
 * N - count of non-zero nodes in tree
 * M - count of snapshots
 * H - tree height
 * Worst case space complexity: O(N * M)
 * Sibling path access: O(H) database reads
 */
export class FullTreeSnapshotBuilder<T extends Bufferable>
  extends BaseFullTreeSnapshotBuilder<TreeBase<T>, TreeSnapshot<T>>
  implements TreeSnapshotBuilder<TreeSnapshot<T>>
{
  constructor(db: AztecKVStore, tree: TreeBase<T>, private deserializer: FromBuffer<T>) {
    super(db, tree);
  }

  protected openSnapshot(root: Buffer, numLeaves: bigint): TreeSnapshot<T> {
    return new BaseFullTreeSnapshot(this.nodes, root, numLeaves, this.tree, this.deserializer);
  }
}
