import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple } from '@aztec/foundation/serialize';

import { MAX_NEW_COMMITMENTS_PER_CALL, PRIVATE_DATA_TREE_HEIGHT } from '../cbind/constants.gen.js';
import { makeTuple, range } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { MembershipWitness } from './membership_witness.js';

/**
 * A ReadRequestMembershipWitness is similar to a MembershipWitness but includes
 * some additional fields used to direct the kernel regarding whether a read is transient
 * and if so which commitment it corresponds to.
 */
export class ReadRequestMembershipWitness {
  constructor(
    /**
     * Index of a leaf in the Merkle tree.
     */
    public leafIndex: Fr,
    /**
     * Sibling path of the leaf in the Merkle tree.
     */
    public siblingPath: Tuple<Fr, typeof PRIVATE_DATA_TREE_HEIGHT>,
    /**
     * Whether or not the read request corresponds to a pending commitment.
     */
    public isTransient = false,
    /**
     * When transient, the commitment being read was created by some app circuit in the current TX.
     * The kernel will need some hint to efficiently find that commitment for a given read request.
     * When not transient, this can be 0.
     */
    public hintToCommitment: Fr,
  ) {
    if (hintToCommitment.value > MAX_NEW_COMMITMENTS_PER_CALL) {
      throw new Error(
        `Expected ReadRequestMembershipWitness' hintToCommitment(${hintToCommitment}) to be <= NEW_COMMITMENTS_LENGTH(${MAX_NEW_COMMITMENTS_PER_CALL})`,
      );
    }
  }

  toBuffer() {
    return serializeToBuffer(
      toBufferBE(this.leafIndex.toBigInt(), 32),
      ...this.siblingPath,
      this.isTransient,
      this.hintToCommitment,
    );
  }

  static mock(size: number, start: number) {
    return new ReadRequestMembershipWitness(
      new Fr(start),
      range(size, start).map(x => new Fr(BigInt(x))) as Tuple<Fr, typeof PRIVATE_DATA_TREE_HEIGHT>,
      false,
      new Fr(0),
    );
  }

  /**
   * Creates a random membership witness. Used for testing purposes.
   * @returns Random membership witness.
   */
  public static random() {
    return new ReadRequestMembershipWitness(
      new Fr(0n),
      makeTuple(PRIVATE_DATA_TREE_HEIGHT, () => Fr.random()),
      false,
      new Fr(0),
    );
  }

  /**
   * Creates a read request membership witness whose sibling path is full of zero fields.
   * @param leafIndex - Index of the leaf in the Merkle tree.
   * @returns Membership witness with zero sibling path.
   */
  public static empty(leafIndex: bigint): ReadRequestMembershipWitness {
    const arr = makeTuple(PRIVATE_DATA_TREE_HEIGHT, () => Fr.ZERO);
    return new ReadRequestMembershipWitness(new Fr(leafIndex), arr, false, new Fr(0));
  }

  /**
   * Creates a transient read request membership witness.
   * @returns an empty transient read request membership witness.
   */
  public static emptyTransient(): ReadRequestMembershipWitness {
    const arr = makeTuple(PRIVATE_DATA_TREE_HEIGHT, () => Fr.ZERO);
    return new ReadRequestMembershipWitness(new Fr(0), arr, true, new Fr(0));
  }

  static fromBufferArray(
    leafIndex: Fr,
    siblingPath: Tuple<Buffer, typeof PRIVATE_DATA_TREE_HEIGHT>,
    isTransient: boolean,
    hintToCommitment: Fr,
  ): ReadRequestMembershipWitness {
    return new ReadRequestMembershipWitness(
      leafIndex,
      siblingPath.map(x => Fr.fromBuffer(x)) as Tuple<Fr, typeof PRIVATE_DATA_TREE_HEIGHT>,
      isTransient,
      hintToCommitment,
    );
  }

  static fromMembershipWitness(
    membershipWitness: MembershipWitness<typeof PRIVATE_DATA_TREE_HEIGHT>,
    isTransient: boolean,
    hintToCommitment: Fr,
  ): ReadRequestMembershipWitness {
    return new ReadRequestMembershipWitness(
      new Fr(membershipWitness.leafIndex),
      membershipWitness.siblingPath as Tuple<Fr, typeof PRIVATE_DATA_TREE_HEIGHT>,
      isTransient,
      hintToCommitment,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized `ReadRequestMembershipWitness`.
   */
  static fromBuffer(buffer: Buffer | BufferReader): ReadRequestMembershipWitness {
    const reader = BufferReader.asReader(buffer);
    const leafIndex = reader.readFr();
    const siblingPath = reader.readArray<Fr, typeof PRIVATE_DATA_TREE_HEIGHT>(PRIVATE_DATA_TREE_HEIGHT, Fr);
    const isTransient = reader.readBoolean();
    const hintToCommitment = reader.readFr();
    return new ReadRequestMembershipWitness(leafIndex, siblingPath, isTransient, hintToCommitment);
  }
}
