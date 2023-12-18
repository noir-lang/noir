import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { createDebugLogger } from '@aztec/foundation/log';
import { IndexedTreeLeaf, IndexedTreeLeafPreimage } from '@aztec/foundation/trees';
import { Hasher, SiblingPath } from '@aztec/types';

import { LevelUp } from 'levelup';

import {
  BatchInsertionResult,
  IndexedTree,
  IndexedTreeSnapshot,
  IndexedTreeSnapshotBuilder,
  LowLeafWitnessData,
} from '../index.js';
import { TreeBase } from '../tree_base.js';

const log = createDebugLogger('aztec:standard-indexed-tree');

/**
 * Factory for creating leaf preimages.
 */
export interface PreimageFactory {
  /**
   * Creates a new preimage from a leaf.
   * @param leaf - Leaf to create a preimage from.
   * @param nextKey - Next key of the leaf.
   * @param nextIndex - Next index of the leaf.
   */
  fromLeaf(leaf: IndexedTreeLeaf, nextKey: bigint, nextIndex: bigint): IndexedTreeLeafPreimage;
  /**
   * Creates a new preimage from a buffer.
   * @param buffer - Buffer to create a preimage from.
   */
  fromBuffer(buffer: Buffer): IndexedTreeLeafPreimage;
  /**
   * Creates an empty preimage.
   */
  empty(): IndexedTreeLeafPreimage;
  /**
   * Creates a copy of a preimage.
   * @param preimage - Preimage to be cloned.
   */
  clone(preimage: IndexedTreeLeafPreimage): IndexedTreeLeafPreimage;
}

export const buildDbKeyForPreimage = (name: string, index: bigint) => {
  return `${name}:leaf_by_index:${toBufferBE(index, 32).toString('hex')}`;
};

export const buildDbKeyForLeafIndex = (name: string, key: bigint) => {
  return `${name}:leaf_index_by_leaf_key:${toBufferBE(key, 32).toString('hex')}`;
};

/**
 * Factory for creating leaves.
 */
export interface LeafFactory {
  /**
   * Creates a new leaf from a buffer.
   * @param key - Key of the leaf.
   */
  buildDummy(key: bigint): IndexedTreeLeaf;
  /**
   * Creates a new leaf from a buffer.
   * @param buffer - Buffer to create a leaf from.
   */
  fromBuffer(buffer: Buffer): IndexedTreeLeaf;
}

/**
 * Pre-compute empty witness.
 * @param treeHeight - Height of tree for sibling path.
 * @returns An empty witness.
 */
function getEmptyLowLeafWitness<N extends number>(
  treeHeight: N,
  leafPreimageFactory: PreimageFactory,
): LowLeafWitnessData<N> {
  return {
    leafPreimage: leafPreimageFactory.empty(),
    index: 0n,
    siblingPath: new SiblingPath(treeHeight, Array(treeHeight).fill(toBufferBE(0n, 32))),
  };
}

/**
 * Standard implementation of an indexed tree.
 */
export class StandardIndexedTree extends TreeBase implements IndexedTree {
  #snapshotBuilder = new IndexedTreeSnapshotBuilder(this.db, this, this.leafPreimageFactory);
  protected cachedLeafPreimages: { [key: string]: IndexedTreeLeafPreimage } = {};

  public constructor(
    db: LevelUp,
    hasher: Hasher,
    name: string,
    depth: number,
    size: bigint = 0n,
    protected leafPreimageFactory: PreimageFactory,
    protected leafFactory: LeafFactory,
    root?: Buffer,
  ) {
    super(db, hasher, name, depth, size, root);
  }

  /**
   * Appends the given leaves to the tree.
   * @param _leaves - The leaves to append.
   * @returns Empty promise.
   * @remarks Use batchInsert method instead.
   */
  appendLeaves(_leaves: Buffer[]): Promise<void> {
    throw new Error('Not implemented');
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
   * @param includeUncommitted - Indicates whether to include uncommitted leaves in the computation.
   * @returns The value of the leaf at the given index or undefined if the leaf is empty.
   */
  public async getLeafValue(index: bigint, includeUncommitted: boolean): Promise<Buffer | undefined> {
    const preimage = await this.getLatestLeafPreimageCopy(index, includeUncommitted);
    return preimage && preimage.toBuffer();
  }

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newKey - The new key to be inserted into the tree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns The found leaf index and a flag indicating if the corresponding leaf's value is equal to `newValue`.
   */
  async findIndexOfPreviousKey(
    newKey: bigint,
    includeUncommitted: boolean,
  ): Promise<
    | {
        /**
         * The index of the found leaf.
         */
        index: bigint;
        /**
         * A flag indicating if the corresponding leaf's value is equal to `newValue`.
         */
        alreadyPresent: boolean;
      }
    | undefined
  > {
    let lowLeafIndex = await this.getDbLowLeafIndex(newKey);
    let lowLeafPreimage = lowLeafIndex !== undefined ? await this.getDbPreimage(lowLeafIndex) : undefined;

    if (includeUncommitted) {
      const cachedLowLeafIndex = this.getCachedLowLeafIndex(newKey);
      if (cachedLowLeafIndex !== undefined) {
        const cachedLowLeafPreimage = this.getCachedPreimage(cachedLowLeafIndex)!;
        if (!lowLeafPreimage || cachedLowLeafPreimage.getKey() > lowLeafPreimage.getKey()) {
          lowLeafIndex = cachedLowLeafIndex;
          lowLeafPreimage = cachedLowLeafPreimage;
        }
      }
    }

    if (lowLeafIndex === undefined || !lowLeafPreimage) {
      return undefined;
    }

    return {
      index: lowLeafIndex,
      alreadyPresent: lowLeafPreimage.getKey() === newKey,
    };
  }

  private getCachedLowLeafIndex(key: bigint): bigint | undefined {
    const indexes = Object.getOwnPropertyNames(this.cachedLeafPreimages);
    const lowLeafIndexes = indexes
      .map(index => ({
        index: BigInt(index),
        key: this.cachedLeafPreimages[index].getKey(),
      }))
      .filter(({ key: candidateKey }) => candidateKey <= key)
      .sort((a, b) => Number(b.key - a.key));
    return lowLeafIndexes[0]?.index;
  }

  private getCachedLeafIndex(key: bigint): bigint | undefined {
    const index = Object.keys(this.cachedLeafPreimages).find(index => {
      return this.cachedLeafPreimages[index].getKey() === key;
    });
    if (index) {
      return BigInt(index);
    }
    return undefined;
  }

  private async getDbLowLeafIndex(key: bigint): Promise<bigint | undefined> {
    return await new Promise<bigint | undefined>((resolve, reject) => {
      let lowLeafIndex: bigint | undefined;
      this.db
        .createReadStream({
          gte: buildDbKeyForLeafIndex(this.getName(), 0n),
          lte: buildDbKeyForLeafIndex(this.getName(), key),
          limit: 1,
          reverse: true,
        })
        .on('data', data => {
          lowLeafIndex = toBigIntBE(data.value);
        })
        .on('close', function () {})
        .on('end', function () {
          resolve(lowLeafIndex);
        })
        .on('error', function () {
          log.error('stream error');
          reject();
        });
    });
  }

  private async getDbPreimage(index: bigint): Promise<IndexedTreeLeafPreimage | undefined> {
    const dbPreimage = await this.db
      .get(buildDbKeyForPreimage(this.getName(), index))
      .then(data => this.leafPreimageFactory.fromBuffer(data))
      .catch(() => undefined);
    return dbPreimage;
  }

  private getCachedPreimage(index: bigint): IndexedTreeLeafPreimage | undefined {
    return this.cachedLeafPreimages[index.toString()];
  }

  /**
   * Gets the latest LeafPreimage copy.
   * @param index - Index of the leaf of which to obtain the LeafPreimage copy.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns A copy of the leaf preimage at the given index or undefined if the leaf was not found.
   */
  public async getLatestLeafPreimageCopy(
    index: bigint,
    includeUncommitted: boolean,
  ): Promise<IndexedTreeLeafPreimage | undefined> {
    const preimage = !includeUncommitted
      ? await this.getDbPreimage(index)
      : this.getCachedPreimage(index) ?? (await this.getDbPreimage(index));
    return preimage && this.leafPreimageFactory.clone(preimage);
  }

  /**
   * Returns the index of a leaf given its value, or undefined if no leaf with that value is found.
   * @param value - The leaf value to look for.
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   * @returns The index of the first leaf found with a given value (undefined if not found).
   */
  public async findLeafIndex(value: Buffer, includeUncommitted: boolean): Promise<bigint | undefined> {
    const leaf = this.leafFactory.fromBuffer(value);
    let index = await this.db
      .get(buildDbKeyForLeafIndex(this.getName(), leaf.getKey()))
      .then(data => toBigIntBE(data))
      .catch(() => undefined);

    if (includeUncommitted && index === undefined) {
      const cachedIndex = this.getCachedLeafIndex(leaf.getKey());
      index = cachedIndex;
    }
    return index;
  }

  /**
   * Initializes the tree.
   * @param prefilledSize - A number of leaves that are prefilled with values.
   * @returns Empty promise.
   *
   * @remarks Explanation of pre-filling:
   *    There needs to be an initial (0,0,0) leaf in the tree, so that when we insert the first 'proper' leaf, we can
   *    prove that any value greater than 0 doesn't exist in the tree yet. We prefill/pad the tree with "the number of
   *    leaves that are added by one block" so that the first 'proper' block can insert a full subtree.
   *
   *    Without this padding, there would be a leaf (0,0,0) at leaf index 0, making it really difficult to insert e.g.
   *    1024 leaves for the first block, because there's only neat space for 1023 leaves after 0. By padding with 1023
   *    more leaves, we can then insert the first block of 1024 leaves into indices 1024:2047.
   */
  public async init(prefilledSize: number): Promise<void> {
    if (prefilledSize < 1) {
      throw new Error(`Prefilled size must be at least 1!`);
    }

    const leaves: IndexedTreeLeafPreimage[] = [];
    for (let i = 0n; i < prefilledSize; i++) {
      const newLeaf = this.leafFactory.buildDummy(i);
      const newLeafPreimage = this.leafPreimageFactory.fromLeaf(newLeaf, i + 1n, i + 1n);
      leaves.push(newLeafPreimage);
    }

    // Make the last leaf point to the first leaf
    leaves[prefilledSize - 1] = this.leafPreimageFactory.fromLeaf(leaves[prefilledSize - 1].asLeaf(), 0n, 0n);

    await this.encodeAndAppendLeaves(leaves, true);
    await this.commit();
  }

  /**
   * Commits all the leaves to the database and removes them from a cache.
   */
  private async commitLeaves(): Promise<void> {
    const batch = this.db.batch();
    const keys = Object.getOwnPropertyNames(this.cachedLeafPreimages);
    for (const key of keys) {
      const leaf = this.cachedLeafPreimages[key];
      const index = BigInt(key);
      batch.put(buildDbKeyForPreimage(this.getName(), index), leaf.toBuffer());
      batch.put(buildDbKeyForLeafIndex(this.getName(), leaf.getKey()), toBufferBE(index, 32));
    }
    await batch.write();
    this.clearCachedLeaves();
  }

  /**
   * Clears the cache.
   */
  private clearCachedLeaves() {
    this.cachedLeafPreimages = {};
  }

  /**
   * Updates a leaf in the tree.
   * @param preimage - New contents of the leaf.
   * @param index - Index of the leaf to be updated.
   */
  protected async updateLeaf(preimage: IndexedTreeLeafPreimage, index: bigint) {
    if (index > this.maxIndex) {
      throw Error(`Index out of bounds. Index ${index}, max index: ${this.maxIndex}.`);
    }

    this.cachedLeafPreimages[index.toString()] = preimage;
    const encodedLeaf = this.encodeLeaf(preimage, true);
    await this.addLeafToCacheAndHashToRoot(encodedLeaf, index);
    const numLeaves = this.getNumLeaves(true);
    if (index >= numLeaves) {
      this.cachedSize = index + 1n;
    }
  }

  /* eslint-disable jsdoc/require-description-complete-sentence */
  /* The following doc block messes up with complete-sentence, so we just disable it */

  /**
   *
   * Each base rollup needs to provide non membership / inclusion proofs for each of the nullifier.
   * This method will return membership proofs and perform partial node updates that will
   * allow the circuit to incrementally update the tree and perform a batch insertion.
   *
   * This offers massive circuit performance savings over doing incremental insertions.
   *
   * WARNING: This function has side effects, it will insert values into the tree.
   *
   * Assumptions:
   * 1. There are 8 nullifiers provided and they are either unique or empty. (denoted as 0)
   * 2. If kc 0 has 1 nullifier, and kc 1 has 3 nullifiers the layout will assume to be the sparse
   *   nullifier layout: [kc0-0, 0, 0, 0, kc1-0, kc1-1, kc1-2, 0]
   *
   * Algorithm overview
   *
   * In general, if we want to batch insert items, we first need to update their low nullifier to point to them,
   * then batch insert all of the values at once in the final step.
   * To update a low nullifier, we provide an insertion proof that the low nullifier currently exists to the
   * circuit, then update the low nullifier.
   * Updating this low nullifier will in turn change the root of the tree. Therefore future low nullifier insertion proofs
   * must be given against this new root.
   * As a result, each low nullifier membership proof will be provided against an intermediate tree state, each with differing
   * roots.
   *
   * This become tricky when two items that are being batch inserted need to update the same low nullifier, or need to use
   * a value that is part of the same batch insertion as their low nullifier. What we do to avoid this case is to
   * update the existing leaves in the tree with the nullifiers in high to low order, ensuring that this case never occurs.
   * The circuit has to sort the nullifiers (or take a hint of the sorted nullifiers and prove that it's a valid permutation).
   * Then we just batch insert the new nullifiers in the original order.
   *
   * The following example will illustrate attempting to insert 2,3,20,19 into a tree already containing 0,5,10,15
   *
   * The example will explore two cases. In each case the values low nullifier will exist within the batch insertion,
   * One where the low nullifier comes before the item in the set (2,3), and one where it comes after (20,19).
   *
   * First, we sort the nullifiers high to low, that's 20,19,3,2
   *
   * The original tree:                       Pending insertion subtree
   *
   *  index     0       1       2       3         -       -       -       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         -       -       -       -
   *  nextIdx   1       2       3       0         -       -       -       -
   *  nextVal   5      10      15       0         -       -       -       -
   *
   *
   * Inserting 20:
   * 1. Find the low nullifier (3) - provide inclusion proof
   * 2. Update its pointers
   * 3. Insert 20 into the pending subtree
   *
   *  index     0       1       2       3         -       -       6       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         -       -      20       -
   *  nextIdx   1       2       3       6         -       -       0       -
   *  nextVal   5      10      15      20         -       -       0       -
   *
   * Inserting 19:
   * 1. Find the low nullifier (3) - provide inclusion proof
   * 2. Update its pointers
   * 3. Insert 19 into the pending subtree
   *
   *  index     0       1       2       3         -       -       6       7
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         -       -      20      19
   *  nextIdx   1       2       3       7         -       -       0       6
   *  nextVal   5      10      15      19         -       -       0      20
   *
   * Inserting 3:
   * 1. Find the low nullifier (0) - provide inclusion proof
   * 2. Update its pointers
   * 3. Insert 3 into the pending subtree
   *
   *  index     0       1       2       3         -       5       6       7
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         -       3      20      19
   *  nextIdx   5       2       3       7         -       1       0       6
   *  nextVal   3      10      15      19         -       5       0      20
   *
   * Inserting 2:
   * 1. Find the low nullifier (0) - provide inclusion proof
   * 2. Update its pointers
   * 3. Insert 2 into the pending subtree
   *
   *  index     0       1       2       3         4       5       6       7
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       3      20      19
   *  nextIdx   4       2       3       7         5       1       0       6
   *  nextVal   2      10      15      19         3       5       0      20
   *
   * Perform subtree insertion
   *
   *  index     0       1       2       3       4       5       6       7
   *  ---------------------------------------------------------------------
   *  val       0       5      10      15       2       3      20      19
   *  nextIdx   4       2       3       7       5       1       0       6
   *  nextVal   2      10      15      19       3       5       0      20
   *
   * For leaves that allow updating the process is exactly the same. When a leaf is inserted that is already present,
   * the low leaf will be the leaf that is being updated, and it'll get updated and an empty leaf will be inserted instead.
   * For example:
   *
   * Initial state:
   *
   *  index     0       1       2       3        4       5       6       7
   *  ---------------------------------------------------------------------
   *  slot      0       0       0       0        0       0       0       0
   *  value     0       0       0       0        0       0       0       0
   *  nextIdx   0       0       0       0        0       0       0       0
   *  nextSlot  0       0       0       0        0       0       0       0.
   *
   *
   *  Add new value 30:5:
   *
   *  index     0       1       2       3        4       5       6       7
   *  ---------------------------------------------------------------------
   *  slot      0       30      0       0        0       0       0       0
   *  value     0       5       0       0        0       0       0       0
   *  nextIdx   1       0       0       0        0       0       0       0
   *  nextSlot  30      0       0       0        0       0       0       0.
   *
   *
   *  Update the value of 30 to 10 (insert 30:10):
   *
   *  index     0       1       2       3        4       5       6       7
   *  ---------------------------------------------------------------------
   *  slot      0       30      0       0        0       0       0       0
   *  value     0       10      0       0        0       0       0       0
   *  nextIdx   1       0       0       0        0       0       0       0
   *  nextSlot  30      0       0       0        0       0       0       0.
   *
   *  The low leaf is 30, so we update it to 10, and insert an empty leaf at index 2.
   *
   * @param leaves - Values to insert into the tree.
   * @param subtreeHeight - Height of the subtree.
   * @returns The data for the leaves to be updated when inserting the new ones.
   */
  public async batchInsert<
    TreeHeight extends number,
    SubtreeHeight extends number,
    SubtreeSiblingPathHeight extends number,
  >(
    leaves: Buffer[],
    subtreeHeight: SubtreeHeight,
  ): Promise<BatchInsertionResult<TreeHeight, SubtreeSiblingPathHeight>> {
    const insertedKeys = new Map<bigint, boolean>();
    const emptyLowLeafWitness = getEmptyLowLeafWitness(this.getDepth() as TreeHeight, this.leafPreimageFactory);
    // Accumulators
    const lowLeavesWitnesses: LowLeafWitnessData<TreeHeight>[] = leaves.map(() => emptyLowLeafWitness);
    const pendingInsertionSubtree: IndexedTreeLeafPreimage[] = leaves.map(() => this.leafPreimageFactory.empty());

    // Start info
    const startInsertionIndex = this.getNumLeaves(true);

    const leavesToInsert = leaves.map(leaf => this.leafFactory.fromBuffer(leaf));
    const sortedDescendingLeafTuples = leavesToInsert
      .map((leaf, index) => ({ leaf, index }))
      .sort((a, b) => Number(b.leaf.getKey() - a.leaf.getKey()));
    const sortedDescendingLeaves = sortedDescendingLeafTuples.map(leafTuple => leafTuple.leaf);

    // Get insertion path for each leaf
    for (let i = 0; i < leavesToInsert.length; i++) {
      const newLeaf = sortedDescendingLeaves[i];
      const originalIndex = leavesToInsert.indexOf(newLeaf);

      if (newLeaf.isEmpty()) {
        continue;
      }

      if (insertedKeys.has(newLeaf.getKey())) {
        throw new Error('Cannot insert duplicated keys in the same batch');
      } else {
        insertedKeys.set(newLeaf.getKey(), true);
      }

      const indexOfPrevious = await this.findIndexOfPreviousKey(newLeaf.getKey(), true);
      if (indexOfPrevious === undefined) {
        return {
          lowLeavesWitnessData: undefined,
          sortedNewLeaves: sortedDescendingLeafTuples.map(leafTuple => leafTuple.leaf.toBuffer()),
          sortedNewLeavesIndexes: sortedDescendingLeafTuples.map(leafTuple => leafTuple.index),
          newSubtreeSiblingPath: await this.getSubtreeSiblingPath(subtreeHeight, true),
        };
      }

      const isUpdate = indexOfPrevious.alreadyPresent;

      // get the low leaf (existence checked in getting index)
      const lowLeafPreimage = (await this.getLatestLeafPreimageCopy(indexOfPrevious.index, true))!;
      const siblingPath = await this.getSiblingPath<TreeHeight>(BigInt(indexOfPrevious.index), true);

      const witness: LowLeafWitnessData<TreeHeight> = {
        leafPreimage: lowLeafPreimage,
        index: BigInt(indexOfPrevious.index),
        siblingPath,
      };

      // Update the running paths
      lowLeavesWitnesses[i] = witness;

      if (isUpdate) {
        const newLowLeaf = lowLeafPreimage.asLeaf().updateTo(newLeaf);

        const newLowLeafPreimage = this.leafPreimageFactory.fromLeaf(
          newLowLeaf,
          lowLeafPreimage.getNextKey(),
          lowLeafPreimage.getNextIndex(),
        );

        await this.updateLeaf(newLowLeafPreimage, indexOfPrevious.index);

        pendingInsertionSubtree[originalIndex] = this.leafPreimageFactory.empty();
      } else {
        const newLowLeafPreimage = this.leafPreimageFactory.fromLeaf(
          lowLeafPreimage.asLeaf(),
          newLeaf.getKey(),
          startInsertionIndex + BigInt(originalIndex),
        );

        await this.updateLeaf(newLowLeafPreimage, indexOfPrevious.index);

        const currentPendingPreimageLeaf = this.leafPreimageFactory.fromLeaf(
          newLeaf,
          lowLeafPreimage.getNextKey(),
          lowLeafPreimage.getNextIndex(),
        );

        pendingInsertionSubtree[originalIndex] = currentPendingPreimageLeaf;
      }
    }

    const newSubtreeSiblingPath = await this.getSubtreeSiblingPath<SubtreeHeight, SubtreeSiblingPathHeight>(
      subtreeHeight,
      true,
    );

    // Perform batch insertion of new pending values
    // Note: In this case we set `hash0Leaf` param to false because batch insertion algorithm use forced null leaf
    // inclusion. See {@link encodeLeaf} for  a more through param explanation.
    await this.encodeAndAppendLeaves(pendingInsertionSubtree, false);

    return {
      lowLeavesWitnessData: lowLeavesWitnesses,
      sortedNewLeaves: sortedDescendingLeafTuples.map(leafTuple => leafTuple.leaf.toBuffer()),
      sortedNewLeavesIndexes: sortedDescendingLeafTuples.map(leafTuple => leafTuple.index),
      newSubtreeSiblingPath,
    };
  }

  async getSubtreeSiblingPath<SubtreeHeight extends number, SubtreeSiblingPathHeight extends number>(
    subtreeHeight: SubtreeHeight,
    includeUncommitted: boolean,
  ): Promise<SiblingPath<SubtreeSiblingPathHeight>> {
    const nextAvailableLeafIndex = this.getNumLeaves(includeUncommitted);
    const fullSiblingPath = await this.getSiblingPath(nextAvailableLeafIndex, includeUncommitted);

    // Drop the first subtreeHeight items since we only care about the path to the subtree root
    return fullSiblingPath.getSubtreeSiblingPath(subtreeHeight);
  }

  snapshot(blockNumber: number): Promise<IndexedTreeSnapshot> {
    return this.#snapshotBuilder.snapshot(blockNumber);
  }

  getSnapshot(block: number): Promise<IndexedTreeSnapshot> {
    return this.#snapshotBuilder.getSnapshot(block);
  }

  /**
   * Encodes leaves and appends them to a tree.
   * @param preimages - Leaves to encode.
   * @param hash0Leaf - Indicates whether 0 value leaf should be hashed. See {@link encodeLeaf}.
   * @returns Empty promise
   */
  private async encodeAndAppendLeaves(preimages: IndexedTreeLeafPreimage[], hash0Leaf: boolean): Promise<void> {
    const startInsertionIndex = this.getNumLeaves(true);

    const hashedLeaves = preimages.map((preimage, i) => {
      this.cachedLeafPreimages[(startInsertionIndex + BigInt(i)).toString()] = preimage;
      return this.encodeLeaf(preimage, hash0Leaf);
    });

    await super.appendLeaves(hashedLeaves);
  }

  /**
   * Encode a leaf into a buffer.
   * @param leaf - Leaf to encode.
   * @param hash0Leaf - Indicates whether 0 value leaf should be hashed. Not hashing 0 value can represent a forced
   *                    null leaf insertion. Detecting this case by checking for 0 value is safe as in the case of
   *                    nullifier it is improbable that a valid nullifier would be 0.
   * @returns Leaf encoded in a buffer.
   */
  private encodeLeaf(leaf: IndexedTreeLeafPreimage, hash0Leaf: boolean): Buffer {
    let encodedLeaf;
    if (!hash0Leaf && leaf.getKey() == 0n) {
      encodedLeaf = toBufferBE(0n, 32);
    } else {
      encodedLeaf = this.hasher.hashInputs(leaf.toHashInputs());
    }
    return encodedLeaf;
  }
}
