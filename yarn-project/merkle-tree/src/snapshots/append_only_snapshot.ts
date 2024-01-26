import { AztecKVStore, AztecMap } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';
import { SiblingPath } from '@aztec/types/membership';

import { AppendOnlyTree } from '../interfaces/append_only_tree.js';
import { TreeBase } from '../tree_base.js';
import { TreeSnapshot, TreeSnapshotBuilder } from './snapshot_builder.js';

// stores the last block that modified this node
const nodeModifiedAtBlockKey = (level: number, index: bigint) => `node:${level}:${index}:modifiedAtBlock`;

// stores the value of the node at the above block
const historicalNodeKey = (level: number, index: bigint) => `node:${level}:${index}:value`;

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
 * A more space-efficient way of storing snapshots of AppendOnlyTrees that trades space need for slower
 * sibling path reads.
 *
 * Complexity:
 *
 * N - count of non-zero nodes in tree
 * M - count of snapshots
 * H - tree height
 *
 * Space complexity: O(N + M) (N nodes - stores the last snapshot for each node and M - ints, for each snapshot stores up to which leaf its written to)
 * Sibling path access:
 *  Best case: O(H) database reads + O(1) hashes
 *  Worst case: O(H) database reads + O(H) hashes
 */
export class AppendOnlySnapshotBuilder implements TreeSnapshotBuilder {
  #nodeValue: AztecMap<ReturnType<typeof historicalNodeKey>, Buffer>;
  #nodeLastModifiedByBlock: AztecMap<ReturnType<typeof nodeModifiedAtBlockKey>, number>;
  #snapshotMetadata: AztecMap<number, SnapshotMetadata>;

  constructor(private db: AztecKVStore, private tree: TreeBase & AppendOnlyTree, private hasher: Hasher) {
    const treeName = tree.getName();
    this.#nodeValue = db.openMap(`append_only_snapshot:${treeName}:node`);
    this.#nodeLastModifiedByBlock = db.openMap(`append_ony_snapshot:${treeName}:block`);
    this.#snapshotMetadata = db.openMap(`append_only_snapshot:${treeName}:snapshot_metadata`);
  }

  getSnapshot(block: number): Promise<TreeSnapshot> {
    const meta = this.#getSnapshotMeta(block);

    if (typeof meta === 'undefined') {
      return Promise.reject(new Error(`Snapshot for tree ${this.tree.getName()} at block ${block} does not exist`));
    }

    return Promise.resolve(
      new AppendOnlySnapshot(
        this.#nodeValue,
        this.#nodeLastModifiedByBlock,
        block,
        meta.numLeaves,
        meta.root,
        this.tree,
        this.hasher,
      ),
    );
  }

  snapshot(block: number): Promise<TreeSnapshot> {
    return this.db.transaction(() => {
      const meta = this.#getSnapshotMeta(block);
      if (typeof meta !== 'undefined') {
        // no-op, we already have a snapshot
        return new AppendOnlySnapshot(
          this.#nodeValue,
          this.#nodeLastModifiedByBlock,
          block,
          meta.numLeaves,
          meta.root,
          this.tree,
          this.hasher,
        );
      }

      const root = this.tree.getRoot(false);
      const depth = this.tree.getDepth();
      const queue: [Buffer, number, bigint][] = [[root, 0, 0n]];

      // walk the tree in BF and store latest nodes
      while (queue.length > 0) {
        const [node, level, index] = queue.shift()!;

        const historicalValue = this.#nodeValue.get(historicalNodeKey(level, index));
        if (!historicalValue || !node.equals(historicalValue)) {
          // we've never seen this node before or it's different than before
          // update the historical tree and tag it with the block that modified it
          void this.#nodeLastModifiedByBlock.set(nodeModifiedAtBlockKey(level, index), block);
          void this.#nodeValue.set(historicalNodeKey(level, index), node);
        } else {
          // if this node hasn't changed, that means, nothing below it has changed either
          continue;
        }

        if (level + 1 > depth) {
          // short circuit if we've reached the leaf level
          // otherwise getNode might throw if we ask for the children of a leaf
          continue;
        }

        // these could be undefined because zero hashes aren't stored in the tree
        const [lhs, rhs] = [this.tree.getNode(level + 1, 2n * index), this.tree.getNode(level + 1, 2n * index + 1n)];

        if (lhs) {
          queue.push([lhs, level + 1, 2n * index]);
        }

        if (rhs) {
          queue.push([rhs, level + 1, 2n * index + 1n]);
        }
      }

      const numLeaves = this.tree.getNumLeaves(false);
      void this.#snapshotMetadata.set(block, {
        numLeaves,
        root,
      });

      return new AppendOnlySnapshot(
        this.#nodeValue,
        this.#nodeLastModifiedByBlock,
        block,
        numLeaves,
        root,
        this.tree,
        this.hasher,
      );
    });
  }

  #getSnapshotMeta(block: number): SnapshotMetadata | undefined {
    return this.#snapshotMetadata.get(block);
  }
}

/**
 * a
 */
class AppendOnlySnapshot implements TreeSnapshot {
  constructor(
    private nodes: AztecMap<string, Buffer>,
    private nodeHistory: AztecMap<string, number>,
    private block: number,
    private leafCount: bigint,
    private historicalRoot: Buffer,
    private tree: TreeBase & AppendOnlyTree,
    private hasher: Hasher,
  ) {}

  public getSiblingPath<N extends number>(index: bigint): SiblingPath<N> {
    const path: Buffer[] = [];
    const depth = this.tree.getDepth();
    let level = depth;

    while (level > 0) {
      const isRight = index & 0x01n;
      const siblingIndex = isRight ? index - 1n : index + 1n;

      const sibling = this.#getHistoricalNodeValue(level, siblingIndex);
      path.push(sibling);

      level -= 1;
      index >>= 1n;
    }

    return new SiblingPath<N>(depth as N, path);
  }

  getDepth(): number {
    return this.tree.getDepth();
  }

  getNumLeaves(): bigint {
    return this.leafCount;
  }

  getRoot(): Buffer {
    // we could recompute it, but it's way cheaper to just store the root
    return this.historicalRoot;
  }

  getLeafValue(index: bigint): Buffer | undefined {
    const leafLevel = this.getDepth();
    const blockNumber = this.#getBlockNumberThatModifiedNode(leafLevel, index);

    // leaf hasn't been set yet
    if (typeof blockNumber === 'undefined') {
      return undefined;
    }

    // leaf was set some time in the past
    if (blockNumber <= this.block) {
      return this.nodes.get(historicalNodeKey(leafLevel, index));
    }

    // leaf has been set but in a block in the future
    return undefined;
  }

  #getHistoricalNodeValue(level: number, index: bigint): Buffer {
    const blockNumber = this.#getBlockNumberThatModifiedNode(level, index);

    // node has never been set
    if (typeof blockNumber === 'undefined') {
      return this.tree.getZeroHash(level);
    }

    // node was set some time in the past
    if (blockNumber <= this.block) {
      return this.nodes.get(historicalNodeKey(level, index))!;
    }

    // the node has been modified since this snapshot was taken
    // because we're working with an AppendOnly tree, historical leaves never change
    // so what we do instead is rebuild this Merkle path up using zero hashes as needed
    // worst case this will do O(H) hashes
    //
    // we first check if this subtree was touched by the block
    // compare how many leaves this block added to the leaf interval of this subtree
    // if they don't intersect then the whole subtree was a hash of zero
    // if they do then we need to rebuild the merkle tree
    const depth = this.tree.getDepth();
    const leafStart = index * 2n ** BigInt(depth - level);
    if (leafStart >= this.leafCount) {
      return this.tree.getZeroHash(level);
    }

    const [lhs, rhs] = [
      this.#getHistoricalNodeValue(level + 1, 2n * index),
      this.#getHistoricalNodeValue(level + 1, 2n * index + 1n),
    ];

    return this.hasher.hash(lhs, rhs);
  }

  #getBlockNumberThatModifiedNode(level: number, index: bigint): number | undefined {
    return this.nodeHistory.get(nodeModifiedAtBlockKey(level, index));
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
