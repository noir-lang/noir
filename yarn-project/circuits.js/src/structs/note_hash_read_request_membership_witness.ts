import { makeTuple, range } from '@aztec/foundation/array';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_NEW_NOTE_HASHES_PER_CALL, NOTE_HASH_TREE_HEIGHT } from '../constants.gen.js';
import { type MembershipWitness } from './membership_witness.js';

/**
 * A ReadRequestMembershipWitness is similar to a MembershipWitness but includes
 * some additional fields used to direct the kernel regarding whether a read is transient
 * and if so which commitment it corresponds to.
 */
export class NoteHashReadRequestMembershipWitness {
  constructor(
    /**
     * Index of a leaf in the Merkle tree.
     */
    public leafIndex: Fr,
    /**
     * Sibling path of the leaf in the Merkle tree.
     */
    public siblingPath: Tuple<Fr, typeof NOTE_HASH_TREE_HEIGHT>,
    /**
     * Whether or not the read request corresponds to a pending note hash.
     */
    public isTransient = false,
    /**
     * When transient, the note hash being read was created by some app circuit in the current TX.
     * The kernel will need some hint to efficiently find that note hash for a given read request.
     * When not transient, this can be 0.
     */
    public hintToNoteHash: Fr,
  ) {
    if (hintToNoteHash.toBigInt() > MAX_NEW_NOTE_HASHES_PER_CALL) {
      throw new Error(
        `Expected ReadRequestMembershipWitness' hintToNoteHash(${hintToNoteHash}) to be <= NEW_NOTE_HASHES_LENGTH(${MAX_NEW_NOTE_HASHES_PER_CALL})`,
      );
    }
  }

  toBuffer() {
    return serializeToBuffer(
      toBufferBE(this.leafIndex.toBigInt(), 32),
      ...this.siblingPath,
      this.isTransient,
      this.hintToNoteHash,
    );
  }

  static mock(size: number, start: number) {
    return new NoteHashReadRequestMembershipWitness(
      new Fr(start),
      range(size, start).map(x => new Fr(BigInt(x))) as Tuple<Fr, typeof NOTE_HASH_TREE_HEIGHT>,
      false,
      new Fr(0),
    );
  }

  /**
   * Creates a random membership witness. Used for testing purposes.
   * @returns Random membership witness.
   */
  public static random() {
    return new NoteHashReadRequestMembershipWitness(
      new Fr(0n),
      makeTuple(NOTE_HASH_TREE_HEIGHT, () => Fr.random()),
      false,
      new Fr(0),
    );
  }

  /**
   * Creates a read request membership witness whose sibling path is full of zero fields.
   * @param leafIndex - Index of the leaf in the Merkle tree.
   * @returns Membership witness with zero sibling path.
   */
  public static empty(leafIndex: bigint): NoteHashReadRequestMembershipWitness {
    const arr = makeTuple(NOTE_HASH_TREE_HEIGHT, () => Fr.ZERO);
    return new NoteHashReadRequestMembershipWitness(new Fr(leafIndex), arr, false, new Fr(0));
  }

  /**
   * Creates a transient read request membership witness.
   * @returns an empty transient read request membership witness.
   */
  public static emptyTransient(): NoteHashReadRequestMembershipWitness {
    const arr = makeTuple(NOTE_HASH_TREE_HEIGHT, () => Fr.ZERO);
    return new NoteHashReadRequestMembershipWitness(new Fr(0), arr, true, new Fr(0));
  }

  static fromBufferArray(
    leafIndex: Fr,
    siblingPath: Tuple<Buffer, typeof NOTE_HASH_TREE_HEIGHT>,
    isTransient: boolean,
    hintToNoteHash: Fr,
  ): NoteHashReadRequestMembershipWitness {
    return new NoteHashReadRequestMembershipWitness(
      leafIndex,
      siblingPath.map(x => Fr.fromBuffer(x)) as Tuple<Fr, typeof NOTE_HASH_TREE_HEIGHT>,
      isTransient,
      hintToNoteHash,
    );
  }

  static fromMembershipWitness(
    membershipWitness: MembershipWitness<typeof NOTE_HASH_TREE_HEIGHT>,
    isTransient: boolean,
    hintToNoteHash: Fr,
  ): NoteHashReadRequestMembershipWitness {
    return new NoteHashReadRequestMembershipWitness(
      new Fr(membershipWitness.leafIndex),
      membershipWitness.siblingPath as Tuple<Fr, typeof NOTE_HASH_TREE_HEIGHT>,
      isTransient,
      hintToNoteHash,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized `ReadRequestMembershipWitness`.
   */
  static fromBuffer(buffer: Buffer | BufferReader): NoteHashReadRequestMembershipWitness {
    const reader = BufferReader.asReader(buffer);
    const leafIndex = Fr.fromBuffer(reader);
    const siblingPath = reader.readArray<Fr, typeof NOTE_HASH_TREE_HEIGHT>(NOTE_HASH_TREE_HEIGHT, Fr);
    const isTransient = reader.readBoolean();
    const hintToNoteHash = Fr.fromBuffer(reader);
    return new NoteHashReadRequestMembershipWitness(leafIndex, siblingPath, isTransient, hintToNoteHash);
  }
}
