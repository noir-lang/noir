import { toBigIntBE, toBufferBE } from '@aztec/foundation';
import { Hasher } from '../hasher.js';
import { IndexedTree, LeafData } from '../interfaces/indexed_tree.js';
import { TreeBase } from '../tree_base.js';

const indexToKeyLeaf = (name: string, index: bigint) => {
  return `${name}:leaf:${index}`;
};

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
export class StandardIndexedTree extends TreeBase implements IndexedTree {
  private leaves: LeafData[] = [];
  private cachedLeaves: { [key: number]: LeafData } = {};

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
    await super.commit();
    await this.commitLeaves();
  }

  /**
   * Rolls back the not-yet-committed changes.
   * @returns Empty promise.
   */
  public async rollback(): Promise<void> {
    await super.rollback();
    this.clearCachedLeaves();
  }

  /**
   * Gets the value of the leaf at the given index.
   * @param index - Index of the leaf of which to obtain the value.
   * @param includeUncommitted include uncommitted leaves in the computation.
   * @returns The value of the leaf at the given index or undefined if the leaf is empty.
   */
  public getLeafValue(index: bigint, includeUncommitted: boolean): Promise<Buffer | undefined> {
    const leaf = this.getLatestLeafDataCopy(Number(index), includeUncommitted);
    if (!leaf) return Promise.resolve(undefined);
    return Promise.resolve(toBufferBE(leaf.value, 32));
  }

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newValue - The new value to be inserted into the tree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns Tuple containing the leaf index and a flag to say if the value is a duplicate.
   */
  public findIndexOfPreviousValue(
    newValue: bigint,
    includeUncommitted: boolean,
  ): { index: number; alreadyPresent: boolean } {
    const numLeaves = this.getNumLeaves(includeUncommitted);
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
   * Gets the latest LeafData copy.
   * @param index - Index of the leaf of which to obtain the LeafData copy.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
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

  /**
   * Appends the given leaf to the tree.
   * @param leaf - The leaf to append.
   * @returns Empty promise.
   */
  private async appendLeaf(leaf: Buffer): Promise<void> {
    const newValue = toBigIntBE(leaf);

    // Special case when appending zero
    if (newValue === 0n) {
      const newSize = (this.cachedSize ?? this.size) + 1n;
      if (newSize - 1n > this.maxIndex) {
        throw Error(`Can't append beyond max index. Max index: ${this.maxIndex}`);
      }
      this.cachedSize = newSize;
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
    const currentSize = this.getNumLeaves(true);
    previousLeafCopy.nextIndex = BigInt(currentSize);
    previousLeafCopy.nextValue = newLeaf.value;
    this.cachedLeaves[Number(currentSize)] = newLeaf;
    this.cachedLeaves[Number(indexOfPrevious.index)] = previousLeafCopy;
    await this._updateLeaf(hashEncodedTreeValue(previousLeafCopy, this.hasher), BigInt(indexOfPrevious.index));
    await this._updateLeaf(hashEncodedTreeValue(newLeaf, this.hasher), this.getNumLeaves(true));
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
   * Initializes the tree.
   * @param prefilledSize - A number of leaves that are prefilled with values.
   * @returns Empty promise.
   */
  public async init(prefilledSize: number): Promise<void> {
    this.leaves.push(initialLeaf);
    await this._updateLeaf(hashEncodedTreeValue(initialLeaf, this.hasher), 0n);

    for (let i = 1; i < prefilledSize; i++) {
      await this.appendLeaf(Buffer.from([i]));
    }

    await this.commit();
  }

  /**
   * Loads Merkle tree data from a database and assigns them to this object.
   */
  public async initFromDb(): Promise<void> {
    const startingIndex = 0n;
    const values: LeafData[] = [];
    const promise = new Promise<void>((resolve, reject) => {
      this.db
        .createReadStream({
          gte: indexToKeyLeaf(this.getName(), startingIndex),
          lte: indexToKeyLeaf(this.getName(), 2n ** BigInt(this.getDepth())),
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
    this.clearCachedLeaves();
  }

  /**
   * Clears the cache.
   */
  private clearCachedLeaves() {
    this.cachedLeaves = {};
  }

  /**
   * Updates a leaf in the tree.
   * @param leaf - New contents of the leaf.
   * @param index - Index of the leaf to be updated.
   */
  // TODO: rename back to updateLeaf once the old updateLeaf is removed
  private async _updateLeaf(leaf: Buffer, index: bigint) {
    if (index > this.maxIndex) {
      throw Error(`Index out of bounds. Index ${index}, max index: ${this.maxIndex}.`);
    }
    await this.addLeafToCacheAndHashToRoot(leaf, index);
    const numLeaves = this.getNumLeaves(true);
    if (index >= numLeaves) {
      this.cachedSize = index + 1n;
    }
  }

  /**
   * Exposes the underlying tree's update leaf method
   * @param leaf - The hash to set at the leaf
   * @param index - The index of the element
   */
  // TODO: remove once the batch insertion functionality is moved here from circuit_block_builder.ts
  public async updateLeaf(leaf: LeafData, index: bigint): Promise<void> {
    let encodedLeaf;
    if (leaf.value == 0n) {
      encodedLeaf = toBufferBE(0n, 32);
    } else {
      encodedLeaf = hashEncodedTreeValue(leaf, this.hasher);
    }
    this.cachedLeaves[Number(index)] = leaf;
    await this._updateLeaf(encodedLeaf, index);
  }
}
