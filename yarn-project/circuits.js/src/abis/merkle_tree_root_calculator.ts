import { pedersenHash } from '@aztec/foundation/crypto';

/**
 * Calculates the root of a merkle tree.
 */
export class MerkleTreeRootCalculator {
  private zeroHashes: Buffer[];

  constructor(private height: number, zeroLeaf = Buffer.alloc(32)) {
    this.zeroHashes = Array.from({ length: height }).reduce(
      (acc: Buffer[], _, i) => [...acc, pedersenHash([acc[i], acc[i]])],
      [zeroLeaf],
    );
  }

  computeTreeRoot(leaves: Buffer[] = []) {
    if (leaves.length === 0) {
      return this.zeroHashes[this.zeroHashes.length - 1];
    }

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
