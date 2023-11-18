import { pedersenHash } from '@aztec/foundation/crypto';

/**
 * Merkle tree calculator.
 */
export class MerkleTreeCalculator {
  private zeroHashes: Buffer[];

  constructor(private height: number, zeroLeaf = Buffer.alloc(32)) {
    this.zeroHashes = Array.from({ length: height }).reduce(
      (acc: Buffer[], _, i) => [...acc, pedersenHash([acc[i], acc[i]])],
      [zeroLeaf],
    );
  }

  computeTree(leaves: Buffer[] = []) {
    if (leaves.length === 0) {
      return [this.zeroHashes[this.zeroHashes.length - 1]];
    }

    let result = leaves.slice();

    for (let i = 0; i < this.height; ++i) {
      const numLeaves = 2 ** (this.height - i);
      const newLeaves: Buffer[] = [];
      for (let j = 0; j < leaves.length / 2; ++j) {
        const l = leaves[j * 2];
        const r = leaves[j * 2 + 1] || this.zeroHashes[i];
        newLeaves[j] = pedersenHash([l, r]);
      }
      result = result.concat(new Array(numLeaves - leaves.length).fill(this.zeroHashes[i]), newLeaves);
      leaves = newLeaves;
    }

    return result;
  }

  computeTreeRoot(leaves: Buffer[] = []) {
    if (leaves.length === 0) {
      return this.zeroHashes[this.zeroHashes.length - 1];
    }

    leaves = leaves.slice();

    for (let i = 0; i < this.height; ++i) {
      let j = 0;
      for (; j < leaves.length / 2; ++j) {
        const l = leaves[j * 2];
        const r = leaves[j * 2 + 1] || this.zeroHashes[i];
        leaves[j] = pedersenHash([l, r]);
      }
      leaves = leaves.slice(0, j);
    }

    return leaves[0];
  }
}
