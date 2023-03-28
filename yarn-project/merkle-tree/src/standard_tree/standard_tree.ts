import { LevelUp, LevelUpChain } from 'levelup';
import { SiblingPath } from '../sibling_path/sibling_path.js';
import { Hasher } from '../hasher.js';
import { MerkleTree } from '../merkle_tree.js';
import { toBufferLE, toBigIntLE } from '@aztec/foundation';

const MAX_DEPTH = 32;

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

/**
 * A Merkle tree implementation that uses a LevelDB database to store the tree.
 */
export class StandardMerkleTree implements MerkleTree {
  /**
   * The value of an 'empty' leaf.
   */
  public static ZERO_ELEMENT = Buffer.from('0000000000000000000000000000000000000000000000000000000000000000', 'hex');
  private root!: Buffer;
  private zeroHashes: Buffer[] = [];
  private cache: { [key: string]: Buffer } = {};
  private cachedSize?: bigint;

  constructor(
    private db: LevelUp,
    private hasher: Hasher,
    private name: string,
    private depth: number,
    private size: bigint = 0n,
    root?: Buffer,
    initialLeafValue = StandardMerkleTree.ZERO_ELEMENT,
  ) {
    if (!(depth >= 1 && depth <= MAX_DEPTH)) {
      throw Error('Bad depth');
    }

    // Compute the zero values at each layer.
    let current = initialLeafValue;
    for (let i = depth - 1; i >= 0; --i) {
      this.zeroHashes[i] = current;
      current = hasher.compress(current, current);
    }

    this.root = root ? root : current;
  }

  /**
   * Creates a new tree.
   * @param db - A database used to store the Merkle tree data.
   * @param hasher - A hasher used to compute hash paths.
   * @param name - Name of the tree.
   * @param depth - Depth of the tree.
   * @param initialLeafValue - The initial value of the leaves.
   * @returns The newly created tree.
   */
  static async new(
    db: LevelUp,
    hasher: Hasher,
    name: string,
    depth: number,
    initialLeafValue = StandardMerkleTree.ZERO_ELEMENT,
  ) {
    const tree = new StandardMerkleTree(db, hasher, name, depth, 0n, undefined, initialLeafValue);
    await tree.writeMeta();
    return tree;
  }

  /**
   * Creates a new tree and sets its root, depth and size based on the meta data which are associated with the name.
   * @param db - A database used to store the Merkle tree data.
   * @param hasher - A hasher used to compute hash paths.
   * @param name - Name of the tree.
   * @param initialLeafValue - The initial value of the leaves before assigned.
   * @returns The newly created tree.
   */
  static async fromName(db: LevelUp, hasher: Hasher, name: string, initialLeafValue = StandardMerkleTree.ZERO_ELEMENT) {
    const meta: Buffer = await db.get(name);
    const { root, depth, size } = decodeMeta(meta);
    return new StandardMerkleTree(db, hasher, name, depth, size, root, initialLeafValue);
  }

  /**
   * Sets the root, depth and size of the tree based on the meta data which are associated with the tree name.
   */
  public async syncFromDb() {
    const meta: Buffer | undefined = await this.dbGet(this.name);
    if (!meta) {
      return;
    }
    const { root, depth, size } = decodeMeta(meta);
    this.root = root;
    this.depth = depth;
    this.size = size;
    this.clearCache();
  }

  /**
   * Returns the root of the tree.
   * @returns The root of the tree.
   */
  public getRoot(): Buffer {
    return this.cache[indexToKeyHash(this.name, 0, 0n)] ?? this.root;
  }

  /**
   * Returns the number of leaves in the tree.
   * @returns The number of leaves in the tree.
   */
  public getNumLeaves() {
    return this.cachedSize ?? this.size;
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
   * @returns A sibling path for the element at the given index.
   * Note: The sibling path is an array of sibling hashes, with the lowest hash (leaf hash) first, and the highest hash last.
   */
  public async getSiblingPath(index: bigint) {
    const path = new SiblingPath();
    let level = this.depth;
    while (level > 0) {
      const isRight = index & 0x01n;
      const sibling = await this.getLatestValueAtIndex(level, isRight ? index - 1n : index + 1n);
      path.data.push(sibling);
      level -= 1;
      index >>= 1n;
    }
    return path;
  }

  /**
   * Appends the given leaves to the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   */
  public async appendLeaves(leaves: Buffer[]): Promise<void> {
    const numLeaves = this.getNumLeaves();
    for (let i = 0; i < leaves.length; i++) {
      const index = numLeaves + BigInt(i);
      await this.addLeafToCacheAndHashToRoot(leaves[i], index);
    }
    this.cachedSize = numLeaves + BigInt(leaves.length);
  }

  /**
   * Updates a leaf in the tree.
   * @param leaf - New contents of the leaf.
   * @param index - Index of the leaf to be updated.
   */
  public async updateLeaf(leaf: Buffer, index: bigint) {
    await this.addLeafToCacheAndHashToRoot(leaf, index);
    const numLeaves = this.getNumLeaves();
    if (index >= numLeaves) {
      this.cachedSize = index + 1n;
    }
  }

  /**
   * Commits the changes to the database.
   * @returns Empty promise.
   */
  public async commit(): Promise<void> {
    const batch = this.db.batch();
    const keys = Object.getOwnPropertyNames(this.cache);
    for (const key of keys) {
      batch.put(key, this.cache[key]);
    }
    this.size = this.getNumLeaves();
    this.root = this.getRoot();
    await this.writeMeta(batch);
    await batch.write();
    this.clearCache();
  }

  /**
   * Rolls back the not-yet-committed changes.
   * @returns Empty promise.
   */
  public rollback(): Promise<void> {
    this.clearCache();
    return Promise.resolve();
  }

  public async getLeafValue(index: bigint): Promise<Buffer | undefined> {
    return this.getLatestValueAtIndex(this.depth, index);
  }

  /**
   * Clears the catch.
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
  private async addLeafToCacheAndHashToRoot(leaf: Buffer, index: bigint) {
    const key = indexToKeyHash(this.name, this.depth, index);
    let current = leaf;
    this.cache[key] = current;
    let level = this.depth;
    while (level > 0) {
      const isRight = index & 0x01n;
      const sibling = await this.getLatestValueAtIndex(level, isRight ? index - 1n : index + 1n);
      const lhs = isRight ? sibling : current;
      const rhs = isRight ? current : sibling;
      current = this.hasher.compress(lhs, rhs);
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
   * @returns The latest value at the given index.
   * Note: If the value is not in the cache, it will be fetched from the database.
   */
  private async getLatestValueAtIndex(level: number, index: bigint): Promise<Buffer> {
    const key = indexToKeyHash(this.name, level, index);
    if (this.cache[key] !== undefined) {
      return this.cache[key];
    }
    const committed = await this.dbGet(key);
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
  private async dbGet(key: string): Promise<Buffer | undefined> {
    return await this.db.get(key).catch(() => {});
  }

  /**
   * Writes meta data to the provided batch.
   * @param batch - The batch to which to write the meta data.
   */
  private async writeMeta(batch?: LevelUpChain<string, Buffer>) {
    const data = encodeMeta(this.getRoot(), this.depth, this.getNumLeaves());
    if (batch) {
      batch.put(this.name, data);
    } else {
      await this.db.put(this.name, data);
    }
  }
}
