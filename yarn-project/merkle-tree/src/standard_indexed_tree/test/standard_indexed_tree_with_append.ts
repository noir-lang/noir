import { StandardIndexedTree } from '../../index.js';

/**
 * A testing utility which is here to store the original implementation of StandardIndexedTree.appendLeaves method
 * that was replaced by the more efficient batchInsert method. We keep the original implementation around as it useful
 * for testing that the more complex batchInsert method works correctly.
 */
export class StandardIndexedTreeWithAppend extends StandardIndexedTree {
  /**
   * Appends the given leaves to the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   * @remarks This method is inefficient and is here mostly for testing. Use batchInsert instead.
   */
  public async appendLeaves(leaves: Buffer[]): Promise<void> {
    for (const leaf of leaves) {
      await this.appendLeaf(leaf);
    }
  }

  /**
   * Appends the given leaf to the tree.
   * @param leaf - The leaf to append.
   * @returns Empty promise.
   */
  private async appendLeaf(leaf: Buffer): Promise<void> {
    const newLeaf = this.leafFactory.fromBuffer(leaf);

    // Special case when appending zero
    if (newLeaf.getKey() === 0n) {
      const newSize = (this.cachedSize ?? this.size) + 1n;
      if (newSize - 1n > this.maxIndex) {
        throw Error(`Can't append beyond max index. Max index: ${this.maxIndex}`);
      }
      this.cachedSize = newSize;
      return;
    }

    const lowLeafIndex = await this.findIndexOfPreviousKey(newLeaf.getKey(), true);
    if (lowLeafIndex === undefined) {
      throw new Error(`Previous leaf not found!`);
    }
    const lowLeafPreimage = (await this.getLatestLeafPreimageCopy(lowLeafIndex.index, true))!;

    const newLeafPreimage = this.leafPreimageFactory.fromLeaf(
      newLeaf,
      lowLeafPreimage.getNextKey(),
      lowLeafPreimage.getNextIndex(),
    );

    if (lowLeafIndex.alreadyPresent) {
      return;
    }
    // insert a new leaf at the highest index and update the values of our previous leaf copy
    const currentSize = this.getNumLeaves(true);
    const newLowLeafPreimage = this.leafPreimageFactory.fromLeaf(
      lowLeafPreimage.asLeaf(),
      newLeaf.getKey(),
      BigInt(currentSize),
    );
    this.cachedLeafPreimages[Number(currentSize)] = newLeafPreimage;
    this.cachedLeafPreimages[Number(lowLeafIndex.index)] = newLowLeafPreimage;
    await this.updateLeaf(newLowLeafPreimage, BigInt(lowLeafIndex.index));
    await this.updateLeaf(newLeafPreimage, this.getNumLeaves(true));
  }
}
