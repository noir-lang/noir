import { assertMemberLength } from '@aztec/foundation/array';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

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
    public siblingPath: Tuple<Fr, N>,
  ) {
    assertMemberLength(this, 'siblingPath', pathSize);
  }

  toBuffer() {
    return serializeToBuffer(toBufferBE(this.leafIndex, 32), ...this.siblingPath);
  }

  /**
   * Creates a random membership witness. Used for testing purposes.
   * @param pathSize - Number of fields in the sibling path.
   * @returns Random membership witness.
   */
  public static random<N extends number>(pathSize: N) {
    return new MembershipWitness<N>(
      pathSize,
      0n,
      Array(pathSize)
        .fill(0)
        .map(() => Fr.random()) as Tuple<Fr, N>,
    );
  }

  /**
   * Creates a membership witness whose sibling path is full of zero fields.
   * @param pathSize - Number of fields in the sibling path.
   * @param leafIndex - Index of the leaf in the Merkle tree.
   * @returns Membership witness with zero sibling path.
   */
  public static empty<N extends number>(pathSize: N, leafIndex = 0n): MembershipWitness<N> {
    const arr = Array(pathSize)
      .fill(0)
      .map(() => Fr.ZERO) as Tuple<Fr, N>;
    return new MembershipWitness<N>(pathSize, leafIndex, arr);
  }

  static fromBufferArray<N extends number>(leafIndex: bigint, siblingPath: Tuple<Buffer, N>): MembershipWitness<N> {
    return new MembershipWitness<N>(
      siblingPath.length as N,
      leafIndex,
      siblingPath.map(x => Fr.fromBuffer(x)) as Tuple<Fr, N>,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized `MembershipWitness`.
   */
  static fromBuffer<N extends number>(buffer: Buffer | BufferReader, size: N): MembershipWitness<N> {
    const reader = BufferReader.asReader(buffer);
    const leafIndex = toBigIntBE(reader.readBytes(32));
    const siblingPath = reader.readArray(size, Fr);
    return new MembershipWitness(size, leafIndex, siblingPath);
  }

  /**
   * Creates a deserializer object for a MembershipWitness with a given size.
   * @param size - Expected size of the witness.
   * @returns A deserializer object.
   */
  static deserializer<N extends number>(size: N): { fromBuffer(buffer: Buffer | BufferReader): MembershipWitness<N> } {
    return {
      fromBuffer: (buffer: Buffer | BufferReader) => {
        const reader = BufferReader.asReader(buffer);
        const leafIndex = toBigIntBE(reader.readBytes(32));
        const siblingPath = reader.readArray(size, Fr);
        return new MembershipWitness(size, leafIndex, siblingPath);
      },
    };
  }

  // import { SiblingPath } from '@aztec/merkle-tree';
  //   static fromSiblingPath<N extends number>(leafIndex: bigint, siblingPath: SiblingPath<N>): MembershipWitness<N> {
  //     return new MembershipWitness<N>(siblingPath.pathSize, leafIndex, siblingPath.toFields() as Tuple<Fr, N>);
  //   }
}
