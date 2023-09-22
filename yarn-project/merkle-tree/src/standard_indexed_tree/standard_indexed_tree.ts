import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { createDebugLogger } from '@aztec/foundation/log';
import { SiblingPath } from '@aztec/types';

import { IndexedTree, LeafData } from '../interfaces/indexed_tree.js';
import { TreeBase } from '../tree_base.js';

const log = createDebugLogger('aztec:standard-indexed-tree');

const indexToKeyLeaf = (name: string, index: bigint) => {
  return `${name}:leaf:${index}`;
};

const zeroLeaf: LeafData = {
  value: 0n,
  nextValue: 0n,
  nextIndex: 0n,
};

/**
 * All of the data to be return during batch insertion.
 */
export interface LowLeafWitnessData<N extends number> {
  /**
   * Preimage of the low nullifier that proves non membership.
   */
  leafData: LeafData;
  /**
   * Sibling path to prove membership of low nullifier.
   */
  siblingPath: SiblingPath<N>;
  /**
   * The index of low nullifier.
   */
  index: bigint;
}

/**
 * Pre-compute empty witness.
 * @param treeHeight - Height of tree for sibling path.
 * @returns An empty witness.
 */
function getEmptyLowLeafWitness<N extends number>(treeHeight: N): LowLeafWitnessData<N> {
  return {
    leafData: zeroLeaf,
    index: 0n,
    siblingPath: new SiblingPath(treeHeight, Array(treeHeight).fill(toBufferBE(0n, 32))),
  };
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const encodeTreeValue = (leafData: LeafData) => {
  const valueAsBuffer = toBufferBE(leafData.value, 32);
  const indexAsBuffer = toBufferBE(leafData.nextIndex, 32);
  const nextValueAsBuffer = toBufferBE(leafData.nextValue, 32);
  return Buffer.concat([valueAsBuffer, indexAsBuffer, nextValueAsBuffer]);
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

/**
 * Indexed merkle tree.
 */
export class StandardIndexedTree extends TreeBase implements IndexedTree {
  protected leaves: LeafData[] = [];
  protected cachedLeaves: { [key: number]: LeafData } = {};

  /**
   * Appends the given leaves to the tree.
   * @param _leaves - The leaves to append.
   * @returns Empty promise.
   * @remarks Use batchInsert method instead.
   */
  public appendLeaves(_leaves: Buffer[]): Promise<void> {
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
  public getLeafValue(index: bigint, includeUncommitted: boolean): Promise<Buffer | undefined> {
    const leaf = this.getLatestLeafDataCopy(Number(index), includeUncommitted);
    if (!leaf) return Promise.resolve(undefined);
    return Promise.resolve(toBufferBE(leaf.value, 32));
  }

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param newValue - The new value to be inserted into the tree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns The found leaf index and a flag indicating if the corresponding leaf's value is equal to `newValue`.
   */
  findIndexOfPreviousValue(
    newValue: bigint,
    includeUncommitted: boolean,
  ): {
    /**
     * The index of the found leaf.
     */
    index: number;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  } {
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

    const leaves: LeafData[] = [];
    for (let i = 0n; i < prefilledSize; i++) {
      const newLeaf = {
        value: toBigIntBE(Buffer.from([Number(i)])),
        nextIndex: i + 1n,
        nextValue: i + 1n,
      };
      leaves.push(newLeaf);
    }

    // Make the first leaf have 0 value
    leaves[0].value = 0n;

    // Make the last leaf point to the first leaf
    leaves[prefilledSize - 1].nextIndex = 0n;
    leaves[prefilledSize - 1].nextValue = 0n;

    await this.encodeAndAppendLeaves(leaves, true);
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
          log.error('stream error');
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
  protected async updateLeaf(leaf: LeafData, index: bigint) {
    if (index > this.maxIndex) {
      throw Error(`Index out of bounds. Index ${index}, max index: ${this.maxIndex}.`);
    }

    const encodedLeaf = this.encodeLeaf(leaf, true);
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
   * A description of the algorithm can be found here: https://colab.research.google.com/drive/1A0gizduSi4FIiIJZ8OylwIpO9-OTqV-R
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
   * a value that is part of the same batch insertion as their low nullifier. In this case a zero low nullifier path is given
   * to the circuit, and it must determine from the set of batch inserted values if the insertion is valid.
   *
   * The following example will illustrate attempting to insert 2,3,20,19 into a tree already containing 0,5,10,15
   *
   * The example will explore two cases. In each case the values low nullifier will exist within the batch insertion,
   * One where the low nullifier comes before the item in the set (2,3), and one where it comes after (20,19).
   *
   * The original tree:                       Pending insertion subtree
   *
   *  index     0       2       3       4         -       -       -       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         -       -       -       -
   *  nextIdx   1       2       3       0         -       -       -       -
   *  nextVal   5      10      15       0         -       -       -       -
   *
   *
   * Inserting 2: (happy path)
   * 1. Find the low nullifier (0) - provide inclusion proof
   * 2. Update its pointers
   * 3. Insert 2 into the pending subtree
   *
   *  index     0       2       3       4         5       -       -       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       -       -       -
   *  nextIdx   5       2       3       0         2       -       -       -
   *  nextVal   2      10      15       0         5       -       -       -
   *
   * Inserting 3: The low nullifier exists within the insertion current subtree
   * 1. When looking for the low nullifier for 3, we will receive 0 again as we have not inserted 2 into the main tree
   *    This is problematic, as we cannot use either 0 or 2 as our inclusion proof.
   *    Why cant we?
   *      - Index 0 has a val 0 and nextVal of 2. This is NOT enough to prove non inclusion of 2.
   *      - Our existing tree is in a state where we cannot prove non inclusion of 3.
   *    We do not provide a non inclusion proof to out circuit, but prompt it to look within the insertion subtree.
   * 2. Update pending insertion subtree
   * 3. Insert 3 into pending subtree
   *
   * (no inclusion proof provided)
   *  index     0       2       3       4         5       6       -       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       3       -       -
   *  nextIdx   5       2       3       0         6       2       -       -
   *  nextVal   2      10      15       0         3       5       -       -
   *
   * Inserting 20: (happy path)
   * 1. Find the low nullifier (15) - provide inclusion proof
   * 2. Update its pointers
   * 3. Insert 20 into the pending subtree
   *
   *  index     0       2       3       4         5       6       7       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       3      20       -
   *  nextIdx   5       2       3       7         6       2       0       -
   *  nextVal   2      10      15      20         3       5       0       -
   *
   * Inserting 19:
   * 1. In this case we can find a low nullifier, but we are updating a low nullifier that has already been updated
   *    We can provide an inclusion proof of this intermediate tree state.
   * 2. Update its pointers
   * 3. Insert 19 into the pending subtree
   *
   *  index     0       2       3       4         5       6       7       8
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       3      20       19
   *  nextIdx   5       2       3       8         6       2       0       7
   *  nextVal   2      10      15      19         3       5       0       20
   *
   * Perform subtree insertion
   *
   *  index     0       2       3       4       5       6       7       8
   *  ---------------------------------------------------------------------
   *  val       0       5      10      15       2       3      20       19
   *  nextIdx   5       2       3       8       6       2       0       7
   *  nextVal   2      10      15      19       3       5       0       20
   *
   * TODO: this implementation will change once the zero value is changed from h(0,0,0). Changes incoming over the next sprint
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
  ): Promise<
    | [LowLeafWitnessData<TreeHeight>[], SiblingPath<SubtreeSiblingPathHeight>]
    | [undefined, SiblingPath<SubtreeSiblingPathHeight>]
  > {
    // Keep track of touched low leaves
    const touched = new Map<number, bigint[]>();

    const emptyLowLeafWitness = getEmptyLowLeafWitness(this.getDepth() as TreeHeight);
    // Accumulators
    const lowLeavesWitnesses: LowLeafWitnessData<TreeHeight>[] = [];
    const pendingInsertionSubtree: LeafData[] = [];

    // Start info
    const startInsertionIndex = this.getNumLeaves(true);

    // Get insertion path for each leaf
    for (let i = 0; i < leaves.length; i++) {
      const newValue = toBigIntBE(leaves[i]);

      // Keep space and just insert zero values
      if (newValue === 0n) {
        pendingInsertionSubtree.push(zeroLeaf);
        lowLeavesWitnesses.push(emptyLowLeafWitness);
        continue;
      }

      const indexOfPrevious = this.findIndexOfPreviousValue(newValue, true);

      // If a touched node has a value that is less than the current value
      const prevNodes = touched.get(indexOfPrevious.index);
      if (prevNodes && prevNodes.some(v => v < newValue)) {
        // check the pending low nullifiers for a low nullifier that works
        // This is the case where the next value is less than the pending
        for (let j = 0; j < pendingInsertionSubtree.length; j++) {
          if (pendingInsertionSubtree[j].value === 0n) continue;

          if (
            pendingInsertionSubtree[j].value < newValue &&
            (pendingInsertionSubtree[j].nextValue > newValue || pendingInsertionSubtree[j].nextValue === 0n)
          ) {
            // add the new value to the pending low nullifiers
            const currentLowLeaf: LeafData = {
              value: newValue,
              nextValue: pendingInsertionSubtree[j].nextValue,
              nextIndex: pendingInsertionSubtree[j].nextIndex,
            };

            pendingInsertionSubtree.push(currentLowLeaf);

            // Update the pending low leaf to point at the new value
            pendingInsertionSubtree[j].nextValue = newValue;
            pendingInsertionSubtree[j].nextIndex = startInsertionIndex + BigInt(i);

            break;
          }
        }

        // Any node updated in this space will need to calculate its low nullifier from a previously inserted value
        lowLeavesWitnesses.push(emptyLowLeafWitness);
      } else {
        // Update the touched mapping
        if (prevNodes) {
          prevNodes.push(newValue);
          touched.set(indexOfPrevious.index, prevNodes);
        } else {
          touched.set(indexOfPrevious.index, [newValue]);
        }

        // get the low leaf
        const lowLeaf = this.getLatestLeafDataCopy(indexOfPrevious.index, true);
        if (lowLeaf === undefined) {
          return [undefined, await this.getSubtreeSiblingPath(subtreeHeight, true)];
        }
        const siblingPath = await this.getSiblingPath<TreeHeight>(BigInt(indexOfPrevious.index), true);

        const witness: LowLeafWitnessData<TreeHeight> = {
          leafData: { ...lowLeaf },
          index: BigInt(indexOfPrevious.index),
          siblingPath,
        };

        // Update the running paths
        lowLeavesWitnesses.push(witness);

        const currentLowLeaf: LeafData = {
          value: newValue,
          nextValue: lowLeaf.nextValue,
          nextIndex: lowLeaf.nextIndex,
        };

        pendingInsertionSubtree.push(currentLowLeaf);

        lowLeaf.nextValue = newValue;
        lowLeaf.nextIndex = startInsertionIndex + BigInt(i);

        const lowLeafIndex = indexOfPrevious.index;
        this.cachedLeaves[lowLeafIndex] = lowLeaf;
        await this.updateLeaf(lowLeaf, BigInt(lowLeafIndex));
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
    return [lowLeavesWitnesses, newSubtreeSiblingPath];
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

  /**
   * Encodes leaves and appends them to a tree.
   * @param leaves - Leaves to encode.
   * @param hash0Leaf - Indicates whether 0 value leaf should be hashed. See {@link encodeLeaf}.
   * @returns Empty promise
   */
  private async encodeAndAppendLeaves(leaves: LeafData[], hash0Leaf: boolean): Promise<void> {
    const startInsertionIndex = Number(this.getNumLeaves(true));

    const serializedLeaves = leaves.map((leaf, i) => {
      this.cachedLeaves[startInsertionIndex + i] = leaf;
      return this.encodeLeaf(leaf, hash0Leaf);
    });

    await super.appendLeaves(serializedLeaves);
  }

  /**
   * Encode a leaf into a buffer.
   * @param leaf - Leaf to encode.
   * @param hash0Leaf - Indicates whether 0 value leaf should be hashed. Not hashing 0 value can represent a forced
   *                    null leaf insertion. Detecting this case by checking for 0 value is safe as in the case of
   *                    nullifier it is improbable that a valid nullifier would be 0.
   * @returns Leaf encoded in a buffer.
   */
  private encodeLeaf(leaf: LeafData, hash0Leaf: boolean): Buffer {
    let encodedLeaf;
    if (!hash0Leaf && leaf.value == 0n) {
      encodedLeaf = toBufferBE(0n, 32);
    } else {
      encodedLeaf = this.hasher.compressInputs(
        [leaf.value, leaf.nextIndex, leaf.nextValue].map(val => toBufferBE(val, 32)),
      );
    }
    return encodedLeaf;
  }
}
