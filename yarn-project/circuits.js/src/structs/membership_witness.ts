import { Fr } from '@aztec/foundation/fields';
import { assertLength, range } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';

/**
 * Contains information which can be used to prove that a leaf is a member of a Merkle tree.
 */
export class MembershipWitness<N extends number> {
  constructor(
    /**
     * Size of the sibling path (number of fields it contains).
     */
    pathSize: N,
    /**
     * Index of a leaf in the Merkle tree.
     */
    public leafIndex: bigint,
    /**
     * Sibling path of the leaf in the Merkle tree.
     */
    public siblingPath: Fr[],
  ) {
    assertLength(this, 'siblingPath', pathSize);
  }

  toBuffer() {
    return serializeToBuffer(toBufferBE(this.leafIndex, 32), ...this.siblingPath);
  }

  static mock(size: number, start: number) {
    return new MembershipWitness(
      size,
      BigInt(start),
      range(size, start).map(x => new Fr(BigInt(x))),
    );
  }

  /**
   * Creates a random membership witness. Used for testing purposes.
   * @param pathSize - Number of fields in the siblin path.
   * @returns Random membership witness.
   */
  public static random<N extends number>(pathSize: N) {
    return new MembershipWitness<N>(
      pathSize,
      0n,
      Array(pathSize)
        .fill(0)
        .map(() => Fr.random()),
    );
  }

  /**
   * Creates a membership witness whose sibling path is full of zero fields.
   * @param pathSize - Number of fields in the sibling path.
   * @param leafIndex - Index of the leaf in the Merkle tree.
   * @returns Membership witness with zero sibling path.
   */
  public static empty<N extends number>(pathSize: N, leafIndex: bigint): MembershipWitness<N> {
    const arr = Array(pathSize)
      .fill(0)
      .map(() => Fr.ZERO);
    return new MembershipWitness<N>(pathSize, leafIndex, arr);
  }

  static fromBufferArray(leafIndex: bigint, siblingPath: Buffer[]) {
    return new MembershipWitness(
      siblingPath.length,
      leafIndex,
      siblingPath.map(x => Fr.fromBuffer(x)),
    );
  }
}
