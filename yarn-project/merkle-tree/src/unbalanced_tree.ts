import { SiblingPath } from '@aztec/circuit-types';
import { type Bufferable, type FromBuffer, serializeToBuffer } from '@aztec/foundation/serialize';
import { type Hasher } from '@aztec/types/interfaces';

import { HasherWithStats } from './hasher_with_stats.js';
import { type MerkleTree } from './interfaces/merkle_tree.js';

const indexToKeyHash = (name: string, level: number, index: bigint) => `${name}:${level}:${index}`;

/**
 * An ephemeral unbalanced Merkle tree implementation.
 * Follows the rollup implementation which greedily hashes pairs of nodes up the tree.
 * Remaining rightmost nodes are shifted up until they can be paired. See proving-state.ts -> findMergeLevel.
 */
export class UnbalancedTree<T extends Bufferable = Buffer> implements MerkleTree<T> {
  // This map stores index and depth -> value
  private cache: { [key: string]: Buffer } = {};
  // This map stores value -> index and depth, since we have variable depth
  private valueCache: { [key: string]: string } = {};
  protected size: bigint = 0n;
  protected readonly maxIndex: bigint;

  protected hasher: HasherWithStats;
  root: Buffer = Buffer.alloc(32);

  public constructor(
    hasher: Hasher,
    private name: string,
    private maxDepth: number = 0,
    protected deserializer: FromBuffer<T>,
  ) {
    this.hasher = new HasherWithStats(hasher);
    this.maxIndex = 2n ** BigInt(this.maxDepth) - 1n;
  }

  /**
   * Returns the root of the tree.
   * @returns The root of the tree.
   */
  public getRoot(): Buffer {
    return this.root;
  }

  /**
   * Returns the number of leaves in the tree.
   * @returns The number of leaves in the tree.
   */
  public getNumLeaves() {
    return this.size;
  }

  /**
   * Returns the max depth of the tree.
   * @returns The max depth of the tree.
   */
  public getDepth(): number {
    return this.maxDepth;
  }

  /**
   * @remark A wonky tree is (currently) only ever ephemeral, so we don't use any db to commit to.
   * The fn must exist to implement MerkleTree however.
   */
  public commit(): Promise<void> {
    throw new Error("Unsupported function - cannot commit on an unbalanced tree as it's always ephemeral.");
    return Promise.resolve();
  }

  /**
   * Rolls back the not-yet-committed changes.
   * @returns Empty promise.
   */
  public rollback(): Promise<void> {
    this.clearCache();
    return Promise.resolve();
  }

  /**
   * Clears the cache.
   */
  private clearCache() {
    this.cache = {};
    this.size = 0n;
  }

  /**
   * @remark A wonky tree can validly have duplicate indices:
   * e.g. 001 (index 1 at level 3) and 01 (index 1 at level 2)
   * So this function cannot reliably give the expected leaf value.
   * We cannot add level as an input as its based on the MerkleTree class's function.
   */
  public getLeafValue(_index: bigint): undefined {
    throw new Error('Unsupported function - cannot get leaf value from an index in an unbalanced tree.');
  }

  /**
   * Returns the index of a leaf given its value, or undefined if no leaf with that value is found.
   * @param leaf - The leaf value to look for.
   * @returns The index of the first leaf found with a given value (undefined if not found).
   * @remark This is NOT the index as inserted, but the index which will be used to calculate path structure.
   */
  public findLeafIndex(value: T): bigint | undefined {
    const key = this.valueCache[serializeToBuffer(value).toString('hex')];
    const [, , index] = key.split(':');
    return BigInt(index);
  }

  /**
   * Returns the first index containing a leaf value after `startIndex`.
   * @param value - The leaf value to look for.
   * @param startIndex - The index to start searching from.
   * @returns The index of the first leaf found with a given value (undefined if not found).
   * @remark This is not really used for a wonky tree, but required to implement MerkleTree.
   */
  public findLeafIndexAfter(value: T, startIndex: bigint): bigint | undefined {
    const index = this.findLeafIndex(value);
    if (!index || index < startIndex) {
      return undefined;
    }
    return index;
  }

  /**
   * Returns the node at the given level and index
   * @param level - The level of the element (root is at level 0).
   * @param index - The index of the element.
   * @returns Leaf or node value, or undefined.
   */
  public getNode(level: number, index: bigint): Buffer | undefined {
    if (level < 0 || level > this.maxDepth) {
      throw Error('Invalid level: ' + level);
    }

    if (index < 0 || index >= this.maxIndex) {
      throw Error('Invalid index: ' + index);
    }

    return this.cache[indexToKeyHash(this.name, level, index)];
  }

  /**
   * Returns a sibling path for the element at the given index.
   * @param value - The value of the element.
   * @returns A sibling path for the element.
   * Note: The sibling path is an array of sibling hashes, with the lowest hash (leaf hash) first, and the highest hash last.
   */
  public getSiblingPath<N extends number>(value: bigint): Promise<SiblingPath<N>> {
    const path: Buffer[] = [];
    const [, depth, _index] = this.valueCache[serializeToBuffer(value).toString('hex')].split(':');
    let level = parseInt(depth, 10);
    let index = BigInt(_index);
    while (level > 0) {
      const isRight = index & 0x01n;
      const key = indexToKeyHash(this.name, level, isRight ? index - 1n : index + 1n);
      const sibling = this.cache[key];
      path.push(sibling);
      level -= 1;
      index >>= 1n;
    }
    return Promise.resolve(new SiblingPath<N>(parseInt(depth, 10) as N, path));
  }

  /**
   * Appends the given leaves to the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   */
  public appendLeaves(leaves: T[]): Promise<void> {
    this.hasher.reset();
    if (this.size != BigInt(0)) {
      throw Error(`Can't re-append to an unbalanced tree. Current has ${this.size} leaves.`);
    }
    if (this.size + BigInt(leaves.length) - 1n > this.maxIndex) {
      throw Error(`Can't append beyond max index. Max index: ${this.maxIndex}`);
    }
    const root = this.batchInsert(leaves);
    this.root = root;

    return Promise.resolve();
  }

  /**
   * Calculates root while adding leaves and nodes to the cache.
   * @param leaves - The leaves to append.
   * @returns Resulting root of the tree.
   */
  private batchInsert(_leaves: T[]): Buffer {
    // If we have an even number of leaves, hash them all in pairs
    // Otherwise, store the final leaf to be shifted up to the next odd sized level
    let [layerWidth, nodeToShift] =
      _leaves.length & 1
        ? [_leaves.length - 1, serializeToBuffer(_leaves[_leaves.length - 1])]
        : [_leaves.length, Buffer.alloc(0)];
    // Allocate this layer's leaves and init the next layer up
    let thisLayer = _leaves.slice(0, layerWidth).map(l => serializeToBuffer(l));
    let nextLayer = [];
    // Store the bottom level leaves
    thisLayer.forEach((leaf, i) => this.storeNode(leaf, this.maxDepth, BigInt(i)));
    for (let i = 0; i < this.maxDepth; i++) {
      for (let j = 0; j < layerWidth; j += 2) {
        // Store the hash of each pair one layer up
        nextLayer[j / 2] = this.hasher.hash(serializeToBuffer(thisLayer[j]), serializeToBuffer(thisLayer[j + 1]));
        this.storeNode(nextLayer[j / 2], this.maxDepth - i - 1, BigInt(j >> 1));
      }
      layerWidth /= 2;
      if (layerWidth & 1) {
        if (nodeToShift.length) {
          // If the next layer has odd length, and we have a node that needs to be shifted up, add it here
          nextLayer.push(serializeToBuffer(nodeToShift));
          this.storeNode(nodeToShift, this.maxDepth - i - 1, BigInt((layerWidth * 2) >> 1));
          layerWidth += 1;
          nodeToShift = Buffer.alloc(0);
        } else {
          // If we don't have a node waiting to be shifted, store the next layer's final node to be shifted
          layerWidth -= 1;
          nodeToShift = nextLayer[layerWidth];
        }
      }
      // reset the layers
      thisLayer = nextLayer;
      nextLayer = [];
    }
    this.size += BigInt(_leaves.length);
    // return the root
    return thisLayer[0];
  }

  /**
   * Stores the given node in the cache.
   * @param value - The value to store.
   * @param depth - The depth of the node in the full tree.
   * @param index - The index of the node at the given depth.
   */
  private storeNode(value: T | Buffer, depth: number, index: bigint) {
    const key = indexToKeyHash(this.name, depth, index);
    this.cache[key] = serializeToBuffer(value);
    this.valueCache[serializeToBuffer(value).toString('hex')] = key;
  }
}
