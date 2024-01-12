import { Hasher } from '@aztec/types/interfaces';
import { SiblingPath } from '@aztec/types/membership';

import { LevelUp } from 'levelup';

import { AppendOnlyTree } from '../interfaces/append_only_tree.js';
import { TreeBase } from '../tree_base.js';
import { TreeSnapshot, TreeSnapshotBuilder } from './snapshot_builder.js';

// stores the last block that modified this node
const nodeModifiedAtBlockKey = (treeName: string, level: number, index: bigint) =>
  `snapshot:node:${treeName}:${level}:${index}:block`;

// stores the value of the node at the above block
const historicalNodeKey = (treeName: string, level: number, index: bigint) =>
  `snapshot:node:${treeName}:${level}:${index}:value`;

// metadata for a snapshot
const snapshotRootKey = (treeName: string, block: number) => `snapshot:root:${treeName}:${block}`;
const snapshotNumLeavesKey = (treeName: string, block: number) => `snapshot:numLeaves:${treeName}:${block}`;

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
  constructor(private db: LevelUp, private tree: TreeBase & AppendOnlyTree, private hasher: Hasher) {}
  async getSnapshot(block: number): Promise<TreeSnapshot> {
    const meta = await this.#getSnapshotMeta(block);

    if (typeof meta === 'undefined') {
      throw new Error(`Snapshot for tree ${this.tree.getName()} at block ${block} does not exist`);
    }

    return new AppendOnlySnapshot(this.db, block, meta.numLeaves, meta.root, this.tree, this.hasher);
  }

  async snapshot(block: number): Promise<TreeSnapshot> {
    const meta = await this.#getSnapshotMeta(block);
    if (typeof meta !== 'undefined') {
      // no-op, we already have a snapshot
      return new AppendOnlySnapshot(this.db, block, meta.numLeaves, meta.root, this.tree, this.hasher);
    }

    const batch = this.db.batch();
    const root = this.tree.getRoot(false);
    const depth = this.tree.getDepth();
    const treeName = this.tree.getName();
    const queue: [Buffer, number, bigint][] = [[root, 0, 0n]];

    // walk the tree in BF and store latest nodes
    while (queue.length > 0) {
      const [node, level, index] = queue.shift()!;

      const historicalValue = await this.db.get(historicalNodeKey(treeName, level, index)).catch(() => undefined);
      if (!historicalValue || !node.equals(historicalValue)) {
        // we've never seen this node before or it's different than before
        // update the historical tree and tag it with the block that modified it
        batch.put(nodeModifiedAtBlockKey(treeName, level, index), String(block));
        batch.put(historicalNodeKey(treeName, level, index), node);
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
      const [lhs, rhs] = await Promise.all([
        this.tree.getNode(level + 1, 2n * index),
        this.tree.getNode(level + 1, 2n * index + 1n),
      ]);

      if (lhs) {
        queue.push([lhs, level + 1, 2n * index]);
      }

      if (rhs) {
        queue.push([rhs, level + 1, 2n * index + 1n]);
      }
    }

    const numLeaves = this.tree.getNumLeaves(false);
    batch.put(snapshotNumLeavesKey(treeName, block), String(numLeaves));
    batch.put(snapshotRootKey(treeName, block), root);
    await batch.write();

    return new AppendOnlySnapshot(this.db, block, numLeaves, root, this.tree, this.hasher);
  }

  async #getSnapshotMeta(block: number): Promise<
    | {
        /** The root of the tree snapshot */
        root: Buffer;
        /** The number of leaves in the tree snapshot */
        numLeaves: bigint;
      }
    | undefined
  > {
    try {
      const treeName = this.tree.getName();
      const root = await this.db.get(snapshotRootKey(treeName, block));
      const numLeaves = BigInt(await this.db.get(snapshotNumLeavesKey(treeName, block)));
      return { root, numLeaves };
    } catch (err) {
      return undefined;
    }
  }
}

/**
 * a
 */
class AppendOnlySnapshot implements TreeSnapshot {
  constructor(
    private db: LevelUp,
    private block: number,
    private leafCount: bigint,
    private historicalRoot: Buffer,
    private tree: TreeBase & AppendOnlyTree,
    private hasher: Hasher,
  ) {}

  public async getSiblingPath<N extends number>(index: bigint): Promise<SiblingPath<N>> {
    const path: Buffer[] = [];
    const depth = this.tree.getDepth();
    let level = depth;

    while (level > 0) {
      const isRight = index & 0x01n;
      const siblingIndex = isRight ? index - 1n : index + 1n;

      const sibling = await this.#getHistoricalNodeValue(level, siblingIndex);
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

  async getLeafValue(index: bigint): Promise<Buffer | undefined> {
    const leafLevel = this.getDepth();
    const blockNumber = await this.#getBlockNumberThatModifiedNode(leafLevel, index);

    // leaf hasn't been set yet
    if (typeof blockNumber === 'undefined') {
      return undefined;
    }

    // leaf was set some time in the past
    if (blockNumber <= this.block) {
      return this.db.get(historicalNodeKey(this.tree.getName(), leafLevel, index));
    }

    // leaf has been set but in a block in the future
    return undefined;
  }

  async #getHistoricalNodeValue(level: number, index: bigint): Promise<Buffer> {
    const blockNumber = await this.#getBlockNumberThatModifiedNode(level, index);

    // node has never been set
    if (typeof blockNumber === 'undefined') {
      return this.tree.getZeroHash(level);
    }

    // node was set some time in the past
    if (blockNumber <= this.block) {
      return this.db.get(historicalNodeKey(this.tree.getName(), level, index));
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

    const [lhs, rhs] = await Promise.all([
      this.#getHistoricalNodeValue(level + 1, 2n * index),
      this.#getHistoricalNodeValue(level + 1, 2n * index + 1n),
    ]);

    return this.hasher.hash(lhs, rhs);
  }

  async #getBlockNumberThatModifiedNode(level: number, index: bigint): Promise<number | undefined> {
    try {
      const value: Buffer | string = await this.db.get(nodeModifiedAtBlockKey(this.tree.getName(), level, index));
      return parseInt(value.toString(), 10);
    } catch (err) {
      return undefined;
    }
  }

  async findLeafIndex(value: Buffer): Promise<bigint | undefined> {
    const numLeaves = this.getNumLeaves();
    for (let i = 0n; i < numLeaves; i++) {
      const currentValue = await this.getLeafValue(i);
      if (currentValue && currentValue.equals(value)) {
        return i;
      }
    }
    return undefined;
  }
}
