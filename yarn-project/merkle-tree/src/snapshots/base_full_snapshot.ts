import { AztecKVStore, AztecMap } from '@aztec/kv-store';
import { SiblingPath } from '@aztec/types/membership';

import { TreeBase } from '../tree_base.js';
import { TreeSnapshot, TreeSnapshotBuilder } from './snapshot_builder.js';

/**
 * Metadata for a snapshot, per block
 */
type SnapshotMetadata = {
  /** The tree root at the time */
  root: Buffer;
  /** The number of filled leaves */
  numLeaves: bigint;
};

/**
 * Builds a full snapshot of a tree. This implementation works for any Merkle tree and stores
 * it in a database in a similar way to how a tree is stored in memory, using pointers.
 *
 * Sharing the same database between versions and trees is recommended as the trees would share
 * structure.
 *
 * Implement the protected method `handleLeaf` to store any additional data you need for each leaf.
 *
 * Complexity:
 * N - count of non-zero nodes in tree
 * M - count of snapshots
 * H - tree height
 * Worst case space complexity: O(N * M)
 * Sibling path access: O(H) database reads
 */
export abstract class BaseFullTreeSnapshotBuilder<T extends TreeBase, S extends TreeSnapshot>
  implements TreeSnapshotBuilder<S>
{
  protected nodes: AztecMap<string, [Buffer, Buffer]>;
  protected snapshotMetadata: AztecMap<number, SnapshotMetadata>;

  constructor(protected db: AztecKVStore, protected tree: T) {
    this.nodes = db.openMap(`full_snapshot:${tree.getName()}:node`);
    this.snapshotMetadata = db.openMap(`full_snapshot:${tree.getName()}:metadata`);
  }

  snapshot(block: number): Promise<S> {
    return this.db.transaction(() => {
      const snapshotMetadata = this.#getSnapshotMeta(block);

      if (snapshotMetadata) {
        return this.openSnapshot(snapshotMetadata.root, snapshotMetadata.numLeaves);
      }

      const root = this.tree.getRoot(false);
      const numLeaves = this.tree.getNumLeaves(false);
      const depth = this.tree.getDepth();
      const queue: [Buffer, number, bigint][] = [[root, 0, 0n]];

      // walk the tree breadth-first and store each of its nodes in the database
      // for each node we save two keys
      //   <node hash>:0 -> <left child's hash>
      //   <node hash>:1 -> <right child's hash>
      while (queue.length > 0) {
        const [node, level, i] = queue.shift()!;
        const nodeKey = node.toString('hex');
        // check if the database already has a child for this tree
        // if it does, then we know we've seen the whole subtree below it before
        // and we don't have to traverse it anymore
        // we use the left child here, but it could be anything that shows we've stored the node before
        if (this.nodes.has(nodeKey)) {
          continue;
        }

        if (level + 1 > depth) {
          // short circuit if we've reached the leaf level
          // otherwise getNode might throw if we ask for the children of a leaf
          this.handleLeaf(i, node);
          continue;
        }

        const [lhs, rhs] = [this.tree.getNode(level + 1, 2n * i), this.tree.getNode(level + 1, 2n * i + 1n)];

        // we want the zero hash at the children's level, not the node's level
        const zeroHash = this.tree.getZeroHash(level + 1);

        void this.nodes.set(nodeKey, [lhs ?? zeroHash, rhs ?? zeroHash]);
        // enqueue the children only if they're not zero hashes
        if (lhs) {
          queue.push([lhs, level + 1, 2n * i]);
        }

        if (rhs) {
          queue.push([rhs, level + 1, 2n * i + 1n]);
        }
      }

      void this.snapshotMetadata.set(block, { root, numLeaves });
      return this.openSnapshot(root, numLeaves);
    });
  }

  protected handleLeaf(_index: bigint, _node: Buffer): void {}

  getSnapshot(version: number): Promise<S> {
    const snapshotMetadata = this.#getSnapshotMeta(version);

    if (!snapshotMetadata) {
      return Promise.reject(new Error(`Version ${version} does not exist for tree ${this.tree.getName()}`));
    }

    return Promise.resolve(this.openSnapshot(snapshotMetadata.root, snapshotMetadata.numLeaves));
  }

  protected abstract openSnapshot(root: Buffer, numLeaves: bigint): S;

  #getSnapshotMeta(block: number): SnapshotMetadata | undefined {
    return this.snapshotMetadata.get(block);
  }
}

/**
 * A source of sibling paths from a snapshot tree
 */
export class BaseFullTreeSnapshot implements TreeSnapshot {
  constructor(
    protected db: AztecMap<string, [Buffer, Buffer]>,
    protected historicRoot: Buffer,
    protected numLeaves: bigint,
    protected tree: TreeBase,
  ) {}

  getSiblingPath<N extends number>(index: bigint): SiblingPath<N> {
    const siblings: Buffer[] = [];

    for (const [_node, sibling] of this.pathFromRootToLeaf(index)) {
      siblings.push(sibling);
    }

    // we got the siblings we were looking for, but they are in root-leaf order
    // reverse them here so we have leaf-root (what SiblingPath expects)
    siblings.reverse();

    return new SiblingPath<N>(this.tree.getDepth() as N, siblings);
  }

  getLeafValue(index: bigint): Buffer | undefined {
    let leafNode: Buffer | undefined = undefined;
    for (const [node, _sibling] of this.pathFromRootToLeaf(index)) {
      leafNode = node;
    }

    return leafNode;
  }

  getDepth(): number {
    return this.tree.getDepth();
  }

  getRoot(): Buffer {
    return this.historicRoot;
  }

  getNumLeaves(): bigint {
    return this.numLeaves;
  }

  protected *pathFromRootToLeaf(leafIndex: bigint) {
    const root = this.historicRoot;
    const pathFromRoot = this.#getPathFromRoot(leafIndex);

    let node: Buffer = root;
    for (let i = 0; i < pathFromRoot.length; i++) {
      // get both children. We'll need both anyway (one to keep track of, the other to walk down to)
      const children: [Buffer, Buffer] = this.db.get(node.toString('hex')) ?? [
        this.tree.getZeroHash(i + 1),
        this.tree.getZeroHash(i + 1),
      ];
      const next = children[pathFromRoot[i]];
      const sibling = children[(pathFromRoot[i] + 1) % 2];

      yield [next, sibling];

      node = next;
    }
  }

  /**
   * Calculates the path from the root to the target leaf. Returns an array of 0s and 1s,
   * each 0 represents walking down a left child and each 1 walking down to the child on the right.
   *
   * @param leafIndex - The target leaf
   * @returns An array of 0s and 1s
   */
  #getPathFromRoot(leafIndex: bigint): ReadonlyArray<0 | 1> {
    const path: Array<0 | 1> = [];
    let level = this.tree.getDepth();
    while (level > 0) {
      path.push(leafIndex & 0x01n ? 1 : 0);
      leafIndex >>= 1n;
      level--;
    }

    path.reverse();
    return path;
  }

  findLeafIndex(value: Buffer): bigint | undefined {
    const numLeaves = this.getNumLeaves();
    for (let i = 0n; i < numLeaves; i++) {
      const currentValue = this.getLeafValue(i);
      if (currentValue && currentValue.equals(value)) {
        return i;
      }
    }
    return undefined;
  }
}
