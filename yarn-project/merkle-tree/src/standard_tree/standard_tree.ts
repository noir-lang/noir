import { AppendOnlyTree } from '../interfaces/append_only_tree.js';
import { TreeBase, indexToKeyHash } from '../tree_base.js';

/**
 * A Merkle tree implementation that uses a LevelDB database to store the tree.
 */
export class StandardTree extends TreeBase implements AppendOnlyTree {
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
  public async appendLeaves(leaves: Buffer[]): Promise<void> {
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
        const lhs = await this.getLatestValueAtIndex(level, index * 2n, true);
        const rhs = await this.getLatestValueAtIndex(level, index * 2n + 1n, true);
        const cacheKey = indexToKeyHash(this.name, level - 1, index);
        this.cache[cacheKey] = this.hasher.compress(lhs, rhs);
      }

      level -= 1;
    }
    this.cachedSize = numLeaves + BigInt(leaves.length);
  }
}
