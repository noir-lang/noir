import { LevelUp } from 'levelup';
import { toBigIntBE, toBufferBE } from '@aztec/foundation';
import { MerkleTree } from '../merkle_tree.js';
import { SiblingPath } from '../sibling_path/sibling_path.js';
import { StandardMerkleTree } from '../standard_tree/standard_tree.js';
import { Hasher } from '../hasher.js';

const indexToKeyLeaf = (name: string, index: bigint) => {
  return `${name}:leaf:${index}`;
};

/**
 * A leaf of a tree.
 */
export interface LeafData {
  /**
   * A value of the leaf.
   */
  value: bigint;
  /**
   * An index of the next leaf.
   */
  nextIndex: bigint;
  /**
   * A value of the next leaf.
   */
  nextValue: bigint;
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const encodeTreeValue = (leafData: LeafData) => {
  const valueAsBuffer = toBufferBE(leafData.value, 32);
  const indexAsBuffer = toBufferBE(leafData.nextIndex, 32);
  const nextValueAsBuffer = toBufferBE(leafData.nextValue, 32);
  return Buffer.concat([valueAsBuffer, indexAsBuffer, nextValueAsBuffer]);
};

const hashEncodedTreeValue = (leaf: LeafData, hasher: Hasher) => {
  return hasher.compressInputs([leaf.value, leaf.nextIndex, leaf.nextValue].map(val => toBufferBE(val, 32)));
};

const decodeTreeValue = (buf: Buffer) => {
  const value = toBigIntBE(buf.subarray(0, 32));
  const nextIndex = toBigIntBE(buf.subarray(32, 64));
  const nextValue = toBigIntBE(buf.subarray(64, 96));
  return {
    value,
    nextIndex,
    nextValue,
  } as LeafData;
};

const initialLeaf: LeafData = {
  value: 0n,
  nextIndex: 0n,
  nextValue: 0n,
};

/**
 * A Merkle tree that supports efficient lookup of leaves by value.
 */
export class IndexedTree implements MerkleTree {
  private leaves: LeafData[] = [];
  private cachedLeaves: { [key: number]: LeafData } = {};
  constructor(private underlying: StandardMerkleTree, private hasher: Hasher, private db: LevelUp) {}

  /**
   * Creates an IndexedTree object.
   * @param db - A database used to store the Merkle tree data.
   * @param hasher - A hasher used to compute hash paths.
   * @param name - A name of the tree.
   * @param depth - A depth of the tree.
   * @param prefilledSize - {optional} A number of leaves that are prefilled with values.
   * @returns A promise with the new Merkle tree.
   */
  public static async new(
    db: LevelUp,
    hasher: Hasher,
    name: string,
    depth: number,
    prefilledSize = 0,
  ): Promise<IndexedTree> {
    const underlying = await StandardMerkleTree.new(db, hasher, name, depth, hashEncodedTreeValue(initialLeaf, hasher));
    const tree = new IndexedTree(underlying, hasher, db);
    await tree.init(prefilledSize);
    return tree;
  }

  /**
   * Creates a new tree and sets its root, depth and size based on the meta data which are associated with the name.
   * @param db - A database used to store the Merkle tree data.
   * @param hasher - A hasher used to compute hash paths.
   * @param name - Name of the tree.
   * @returns The newly created tree.
   */
  static async fromName(db: LevelUp, hasher: Hasher, name: string): Promise<IndexedTree> {
    const underlying = await StandardMerkleTree.fromName(db, hasher, name, hashEncodedTreeValue(initialLeaf, hasher));
    const tree = new IndexedTree(underlying, hasher, db);
    await tree.initFromDb();
    return tree;
  }

  /**
   * Returns an empty leaf of the tree.
   * @returns An empty leaf.
   */
  static initialLeaf(): LeafData {
    return initialLeaf;
  }

  /**
   * Returns the root of the tree.
   * @returns The root of the tree.
   */
  public getRoot(includeUncommitted: boolean): Buffer {
    return this.underlying.getRoot(includeUncommitted);
  }

  /**
   * Returns the depth of the tree.
   * @returns The depth of the tree.
   */
  public getDepth(): number {
    return this.underlying.getDepth();
  }

  /**
   * Returns the number of leaves in the tree.
   * @returns The number of leaves in the tree.
   */
  public getNumLeaves(includeUncommitted: boolean): bigint {
    return this.underlying.getNumLeaves(includeUncommitted);
  }

  /**
   * Appends the given leaves to the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   */
  public async appendLeaves(leaves: Buffer[]): Promise<void> {
    for (const leaf of leaves) {
      await this.appendLeaf(leaf);
    }
  }

  /**
   * Commits the changes to the database.
   * @returns Empty promise.
   */
  public async commit(): Promise<void> {
    await this.underlying.commit();
    await this.commitLeaves();
  }

  /**
   * Rolls back the not-yet-committed changes.
   * @returns Empty promise.
   */
  public async rollback(): Promise<void> {
    await this.underlying.rollback();
    this.rollbackLeaves();
  }

  /**
   * Returns a sibling path for the element at the given index.
   * @param index - The index of the element.
   * @returns A sibling path for the element at the given index.
   * Note: The sibling path is an array of sibling hashes, with the lowest hash (leaf hash) first, and the highest hash last.
   */
  public async getSiblingPath(index: bigint, includeUncommitted: boolean): Promise<SiblingPath> {
    return await this.underlying.getSiblingPath(index, includeUncommitted);
  }

  /**
   * Exposes the underlying tree's update leaf method
   * @param leaf - The hash to set at the leaf
   * @param index - The index of the element
   */
  public async updateLeaf(leaf: LeafData, index: bigint): Promise<void> {
    this.cachedLeaves[Number(index)] = leaf;
    const encodedLeaf = hashEncodedTreeValue(leaf, this.hasher);
    await this.underlying.updateLeaf(encodedLeaf, index);
  }

  /**
   * Special case which will force append zero into the tree by increasing its size
   */
  private appendZero(): void {
    this.underlying.forceAppendEmptyLeaf();
  }

  /**
   * Appends the given leaf to the tree.
   * @param leaf - The leaf to append.
   * @returns Empty promise.
   */
  private async appendLeaf(leaf: Buffer): Promise<void> {
    const newValue = toBigIntBE(leaf);

    // Special case when appending zero
    if (newValue === 0n) {
      this.appendZero();
      return;
    }

    const indexOfPrevious = this.findIndexOfPreviousValue(newValue, true);
    const previousLeafCopy = this.getLatestLeafDataCopy(indexOfPrevious.index, true);

    if (previousLeafCopy === undefined) {
      throw new Error(`Previous leaf not found!`);
    }
    const newLeaf = {
      value: newValue,
      nextIndex: previousLeafCopy.nextIndex,
      nextValue: previousLeafCopy.nextValue,
    } as LeafData;
    if (indexOfPrevious.alreadyPresent) {
      return;
    }
    // insert a new leaf at the highest index and update the values of our previous leaf copy
    const currentSize = this.underlying.getNumLeaves(true);
    previousLeafCopy.nextIndex = BigInt(currentSize);
    previousLeafCopy.nextValue = newLeaf.value;
    this.cachedLeaves[Number(currentSize)] = newLeaf;
    this.cachedLeaves[Number(indexOfPrevious.index)] = previousLeafCopy;
    await this.underlying.updateLeaf(
      hashEncodedTreeValue(previousLeafCopy, this.hasher),
      BigInt(indexOfPrevious.index),
    );
    await this.underlying.appendLeaves([hashEncodedTreeValue(newLeaf, this.hasher)]);
  }

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newValue - The new value to be inserted into the tree.
   * @returns Tuple containing the leaf index and a flag to say if the value is a duplicate.
   */
  public findIndexOfPreviousValue(newValue: bigint, includeUncommitted: boolean) {
    const numLeaves = this.underlying.getNumLeaves(includeUncommitted);
    const diff: bigint[] = [];

    for (let i = 0; i < numLeaves; i++) {
      const storedLeaf = this.getLatestLeafDataCopy(i, includeUncommitted)!;

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
    const minIndex = this.findMinIndex(diff);
    return { index: minIndex, alreadyPresent: false };
  }

  /**
   * Finds the index of the minimum value in an array.
   * @param values - The collection of values to be searched.
   * @returns The index of the minimum value in the array.
   */
  private findMinIndex(values: bigint[]) {
    if (!values.length) {
      return 0;
    }
    let minIndex = 0;
    for (let i = 1; i < values.length; i++) {
      if (values[minIndex] > values[i]) {
        minIndex = i;
      }
    }
    return minIndex;
  }

  /**
   * Saves the initial leaf to this object and saves it to a database.
   */
  private async init(initialSize = 1) {
    this.leaves.push(initialLeaf);
    await this.underlying.appendLeaves([hashEncodedTreeValue(initialLeaf, this.hasher)]);

    for (let i = 1; i < initialSize; i++) {
      await this.appendLeaf(Buffer.from([i]));
    }
    await this.commit();
  }

  /**
   * Loads Merkle tree data from a database and assigns them to this object.
   * @param startingIndex - An index locating a first element of the tree.
   */
  private async initFromDb(startingIndex = 0n): Promise<void> {
    const values: LeafData[] = [];
    const promise = new Promise<void>((resolve, reject) => {
      this.db
        .createReadStream({
          gte: indexToKeyLeaf(this.underlying.getName(), startingIndex),
          lte: indexToKeyLeaf(this.underlying.getName(), 2n ** BigInt(this.underlying.getDepth())),
        })
        .on('data', function (data) {
          const index = Number(data.key);
          values[index] = decodeTreeValue(data.value);
        })
        .on('close', function () {})
        .on('end', function () {
          resolve();
        })
        .on('error', function () {
          console.log('stream error');
          reject();
        });
    });
    await promise;
    this.leaves = values;
  }

  /**
   * Commits all the leaves to the database and removes them from a cache.
   */
  private async commitLeaves(): Promise<void> {
    const batch = this.db.batch();
    const keys = Object.getOwnPropertyNames(this.cachedLeaves);
    for (const key of keys) {
      const index = Number(key);
      batch.put(key, this.cachedLeaves[index]);
      this.leaves[index] = this.cachedLeaves[index];
    }
    await batch.write();
    this.clearCache();
  }

  /**
   * Wipes all the leaves in a cache.
   */
  private rollbackLeaves() {
    this.clearCache();
  }

  /**
   * Clears the cache.
   */
  private clearCache() {
    this.cachedLeaves = {};
  }

  /**
   * Gets the latest LeafData copy.
   * @param index - Index of the leaf of which to obtain the LeafData copy.
   * @returns A copy of the leaf data at the given index or undefined if the leaf was not found.
   */
  public getLatestLeafDataCopy(index: number, includeUncommitted: boolean): LeafData | undefined {
    const leaf = !includeUncommitted ? this.leaves[index] : this.cachedLeaves[index] ?? this.leaves[index];
    return leaf
      ? ({
          value: leaf.value,
          nextIndex: leaf.nextIndex,
          nextValue: leaf.nextValue,
        } as LeafData)
      : undefined;
  }

  public getLeafValue(index: bigint, includeUncommitted: boolean): Promise<Buffer | undefined> {
    const leaf = this.getLatestLeafDataCopy(Number(index), includeUncommitted);
    if (!leaf) return Promise.resolve(undefined);
    return Promise.resolve(toBufferBE(leaf.value, 32));
  }
}
