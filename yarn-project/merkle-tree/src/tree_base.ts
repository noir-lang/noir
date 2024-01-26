import { toBigIntLE, toBufferLE } from '@aztec/foundation/bigint-buffer';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore, AztecMap, AztecSingleton } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';
import { SiblingPath } from '@aztec/types/membership';

import { HasherWithStats } from './hasher_with_stats.js';
import { MerkleTree } from './interfaces/merkle_tree.js';

const MAX_DEPTH = 254;

const indexToKeyHash = (name: string, level: number, index: bigint) => `${name}:${level}:${index}`;
const encodeMeta = (root: Buffer, depth: number, size: bigint) => {
  const data = Buffer.alloc(36);
  root.copy(data);
  data.writeUInt32LE(depth, 32);
  return Buffer.concat([data, toBufferLE(size, 32)]);
};
const decodeMeta = (meta: Buffer) => {
  const root = meta.subarray(0, 32);
  const depth = meta.readUInt32LE(32);
  const size = toBigIntLE(meta.subarray(36));
  return {
    root,
    depth,
    size,
  };
};

const openTreeMetaSingleton = (store: AztecKVStore, treeName: string): AztecSingleton<Buffer> =>
  store.openSingleton(`merkle_tree_${treeName}_meta`);

export const getTreeMeta = (store: AztecKVStore, treeName: string) => {
  const singleton = openTreeMetaSingleton(store, treeName);
  const val = singleton.get();
  if (!val) {
    throw new Error();
  }
  return decodeMeta(val);
};

export const INITIAL_LEAF = Buffer.from('0000000000000000000000000000000000000000000000000000000000000000', 'hex');

/**
 * A Merkle tree implementation that uses a LevelDB database to store the tree.
 */
export abstract class TreeBase implements MerkleTree {
  protected readonly maxIndex: bigint;
  protected cachedSize?: bigint;
  private root!: Buffer;
  private zeroHashes: Buffer[] = [];
  private cache: { [key: string]: Buffer } = {};
  protected log: DebugLogger;
  protected hasher: HasherWithStats;

  private nodes: AztecMap<string, Buffer>;
  private meta: AztecSingleton<Buffer>;

  public constructor(
    protected store: AztecKVStore,
    hasher: Hasher,
    private name: string,
    private depth: number,
    protected size: bigint = 0n,
    root?: Buffer,
  ) {
    if (!(depth >= 1 && depth <= MAX_DEPTH)) {
      throw Error('Invalid depth');
    }

    this.hasher = new HasherWithStats(hasher);
    this.nodes = store.openMap('merkle_tree_' + name);
    this.meta = openTreeMetaSingleton(store, name);

    // Compute the zero values at each layer.
    let current = INITIAL_LEAF;
    for (let i = depth - 1; i >= 0; --i) {
      this.zeroHashes[i] = current;
      current = hasher.hash(current, current);
    }

    this.root = root ? root : current;
    this.maxIndex = 2n ** BigInt(depth) - 1n;

    this.log = createDebugLogger(`aztec:merkle-tree:${name}`);
  }

  /**
   * Returns the root of the tree.
   * @param includeUncommitted - If true, root incorporating uncommitted changes is returned.
   * @returns The root of the tree.
   */
  public getRoot(includeUncommitted: boolean): Buffer {
    return !includeUncommitted ? this.root : this.cache[indexToKeyHash(this.name, 0, 0n)] ?? this.root;
  }

  /**
   * Returns the number of leaves in the tree.
   * @param includeUncommitted - If true, the returned number of leaves includes uncommitted changes.
   * @returns The number of leaves in the tree.
   */
  public getNumLeaves(includeUncommitted: boolean) {
    return !includeUncommitted ? this.size : this.cachedSize ?? this.size;
  }

  /**
   * Returns the name of the tree.
   * @returns The name of the tree.
   */
  public getName(): string {
    return this.name;
  }

  /**
   * Returns the depth of the tree.
   * @returns The depth of the tree.
   */
  public getDepth(): number {
    return this.depth;
  }

  /**
   * Returns a sibling path for the element at the given index.
   * @param index - The index of the element.
   * @param includeUncommitted - Indicates whether to get a sibling path incorporating uncommitted changes.
   * @returns A sibling path for the element at the given index.
   * Note: The sibling path is an array of sibling hashes, with the lowest hash (leaf hash) first, and the highest hash last.
   */
  public getSiblingPath<N extends number>(index: bigint, includeUncommitted: boolean): Promise<SiblingPath<N>> {
    const path: Buffer[] = [];
    let level = this.depth;
    while (level > 0) {
      const isRight = index & 0x01n;
      const sibling = this.getLatestValueAtIndex(level, isRight ? index - 1n : index + 1n, includeUncommitted);
      path.push(sibling);
      level -= 1;
      index >>= 1n;
    }
    return Promise.resolve(new SiblingPath<N>(this.depth as N, path));
  }

  /**
   * Commits the changes to the database.
   * @returns Empty promise.
   */
  public commit(): Promise<void> {
    return this.store.transaction(() => {
      const keys = Object.getOwnPropertyNames(this.cache);
      for (const key of keys) {
        void this.nodes.set(key, this.cache[key]);
      }
      this.size = this.getNumLeaves(true);
      this.root = this.getRoot(true);

      this.clearCache();

      void this.writeMeta();
    });
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
   * Gets the value at the given index.
   * @param index - The index of the leaf.
   * @param includeUncommitted - Indicates whether to include uncommitted changes.
   * @returns Leaf value at the given index or undefined.
   */
  public getLeafValue(index: bigint, includeUncommitted: boolean): Buffer | undefined {
    return this.getLatestValueAtIndex(this.depth, index, includeUncommitted);
  }

  public getNode(level: number, index: bigint): Buffer | undefined {
    if (level < 0 || level > this.depth) {
      throw Error('Invalid level: ' + level);
    }

    if (index < 0 || index >= 2n ** BigInt(level)) {
      throw Error('Invalid index: ' + index);
    }

    return this.dbGet(indexToKeyHash(this.name, level, index));
  }

  public getZeroHash(level: number): Buffer {
    if (level <= 0 || level > this.depth) {
      throw new Error('Invalid level');
    }

    return this.zeroHashes[level - 1];
  }

  /**
   * Clears the cache.
   */
  private clearCache() {
    this.cache = {};
    this.cachedSize = undefined;
  }

  /**
   * Adds a leaf and all the hashes above it to the cache.
   * @param leaf - Leaf to add to cache.
   * @param index - Index of the leaf (used to derive the cache key).
   */
  protected addLeafToCacheAndHashToRoot(leaf: Buffer, index: bigint) {
    const key = indexToKeyHash(this.name, this.depth, index);
    let current = leaf;
    this.cache[key] = current;
    let level = this.depth;
    while (level > 0) {
      const isRight = index & 0x01n;
      const sibling = this.getLatestValueAtIndex(level, isRight ? index - 1n : index + 1n, true);
      const lhs = isRight ? sibling : current;
      const rhs = isRight ? current : sibling;
      current = this.hasher.hash(lhs, rhs);
      level -= 1;
      index >>= 1n;
      const cacheKey = indexToKeyHash(this.name, level, index);
      this.cache[cacheKey] = current;
    }
  }

  /**
   * Returns the latest value at the given index.
   * @param level - The level of the tree.
   * @param index - The index of the element.
   * @param includeUncommitted - Indicates, whether to get include uncommitted changes.
   * @returns The latest value at the given index.
   * Note: If the value is not in the cache, it will be fetched from the database.
   */
  private getLatestValueAtIndex(level: number, index: bigint, includeUncommitted: boolean): Buffer {
    const key = indexToKeyHash(this.name, level, index);
    if (includeUncommitted && this.cache[key] !== undefined) {
      return this.cache[key];
    }
    const committed = this.dbGet(key);
    if (committed !== undefined) {
      return committed;
    }
    return this.zeroHashes[level - 1];
  }

  /**
   * Gets a value from db by key.
   * @param key - The key to by which to get the value.
   * @returns A value from the db based on the key.
   */
  private dbGet(key: string): Buffer | undefined {
    return this.nodes.get(key);
  }

  /**
   * Initializes the tree.
   * @param prefilledSize - A number of leaves that are prefilled with values.
   * @returns Empty promise.
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  public async init(prefilledSize: number): Promise<void> {
    // prefilledSize is used only by Indexed Tree.
    await this.writeMeta();
  }

  /**
   * Writes meta data to the provided batch.
   * @param batch - The batch to which to write the meta data.
   */
  protected writeMeta() {
    const data = encodeMeta(this.getRoot(true), this.depth, this.getNumLeaves(true));
    return this.meta.set(data);
  }

  /**
   * Appends the given leaves to the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   *
   * @remarks The batch insertion algorithm works as follows:
   *          1. Insert all the leaves,
   *          2. start iterating over levels from the bottom up,
   *          3. on each level iterate over all the affected nodes (i.e. nodes whose preimages have changed),
   *          4. fetch the preimage, hash it and insert the updated value.
   * @remarks This algorithm is optimal when it comes to the number of hashing operations. It might not be optimal when
   *          it comes to the number of database reads, but that should be irrelevant given that most of the time
   *          `getLatestValueAtIndex` will return a value from cache (because at least one of the 2 children was
   *          touched in previous iteration).
   */
  protected appendLeaves(leaves: Buffer[]): void {
    const numLeaves = this.getNumLeaves(true);
    if (numLeaves + BigInt(leaves.length) - 1n > this.maxIndex) {
      throw Error(`Can't append beyond max index. Max index: ${this.maxIndex}`);
    }

    // 1. Insert all the leaves
    let firstIndex = numLeaves;
    let level = this.depth;
    for (let i = 0; i < leaves.length; i++) {
      const cacheKey = indexToKeyHash(this.name, level, firstIndex + BigInt(i));
      this.cache[cacheKey] = leaves[i];
    }

    let lastIndex = firstIndex + BigInt(leaves.length);
    // 2. Iterate over all the levels from the bottom up
    while (level > 0) {
      firstIndex >>= 1n;
      lastIndex >>= 1n;
      // 3.Iterate over all the affected nodes at this level and update them
      for (let index = firstIndex; index <= lastIndex; index++) {
        const lhs = this.getLatestValueAtIndex(level, index * 2n, true);
        const rhs = this.getLatestValueAtIndex(level, index * 2n + 1n, true);
        const cacheKey = indexToKeyHash(this.name, level - 1, index);
        this.cache[cacheKey] = this.hasher.hash(lhs, rhs);
      }

      level -= 1;
    }
    this.cachedSize = numLeaves + BigInt(leaves.length);
  }

  /**
   * Returns the index of a leaf given its value, or undefined if no leaf with that value is found.
   * @param value - The leaf value to look for.
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   * @returns The index of the first leaf found with a given value (undefined if not found).
   */
  abstract findLeafIndex(value: Buffer, includeUncommitted: boolean): bigint | undefined;
}
