import { makeTuple } from '@aztec/foundation/array';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

import {
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  NULLIFIER_TREE_HEIGHT,
} from '../constants.gen.js';
import { MembershipWitness } from './membership_witness.js';
import { NullifierLeafPreimage } from './rollup/nullifier_leaf/index.js';
import { SideEffectLinkedToNoteHash, type SideEffectType } from './side_effects.js';

export class NonMembershipHint<TREE_HEIGHT extends number, LEAF_PREIMAGE extends IndexedTreeLeafPreimage> {
  constructor(public membershipWitness: MembershipWitness<TREE_HEIGHT>, public leafPreimage: LEAF_PREIMAGE) {}

  static empty<TREE_HEIGHT extends number, LEAF_PREIMAGE extends IndexedTreeLeafPreimage>(
    treeHeight: TREE_HEIGHT,
    makeEmptyLeafPreimage: () => LEAF_PREIMAGE,
  ) {
    return new NonMembershipHint(MembershipWitness.empty(treeHeight, 0n), makeEmptyLeafPreimage());
  }

  static fromBuffer<TREE_HEIGHT extends number, LEAF_PREIMAGE extends IndexedTreeLeafPreimage>(
    buffer: Buffer | BufferReader,
    treeHeight: TREE_HEIGHT,
    leafPreimageFromBuffer: { fromBuffer: (buffer: BufferReader) => LEAF_PREIMAGE },
  ): NonMembershipHint<TREE_HEIGHT, LEAF_PREIMAGE> {
    const reader = BufferReader.asReader(buffer);
    return new NonMembershipHint(
      MembershipWitness.fromBuffer(reader, treeHeight),
      reader.readObject(leafPreimageFromBuffer),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.membershipWitness, this.leafPreimage);
  }
}

export class NonExistentReadRequestHints<
  READ_REQUEST_LEN extends number,
  TREE_HEIGHT extends number,
  LEAF_PREIMAGE extends IndexedTreeLeafPreimage,
  PENDING_VALUE_LEN extends number,
  PENDING_VALUE extends SideEffectType,
> {
  constructor(
    /**
     * The hints for the low leaves of the read requests.
     */
    public nonMembershipHints: Tuple<NonMembershipHint<TREE_HEIGHT, LEAF_PREIMAGE>, READ_REQUEST_LEN>,
    /**
     * Indices of the smallest pending values greater than the read requests.
     */
    public nextPendingValueIndices: Tuple<number, READ_REQUEST_LEN>,
    public sortedPendingValues: Tuple<PENDING_VALUE, PENDING_VALUE_LEN>,
    public sortedPendingValueHints: Tuple<number, PENDING_VALUE_LEN>,
  ) {}

  static fromBuffer<
    READ_REQUEST_LEN extends number,
    TREE_HEIGHT extends number,
    LEAF_PREIMAGE extends IndexedTreeLeafPreimage,
    PENDING_VALUE_LEN extends number,
    PENDING_VALUE extends SideEffectType,
  >(
    buffer: Buffer | BufferReader,
    readRequestLen: READ_REQUEST_LEN,
    treeHeight: TREE_HEIGHT,
    leafPreimageFromBuffer: { fromBuffer: (buffer: BufferReader) => LEAF_PREIMAGE },
    pendingValueLen: PENDING_VALUE_LEN,
    orderedValueFromBuffer: { fromBuffer: (buffer: BufferReader) => PENDING_VALUE },
  ): NonExistentReadRequestHints<READ_REQUEST_LEN, TREE_HEIGHT, LEAF_PREIMAGE, PENDING_VALUE_LEN, PENDING_VALUE> {
    const reader = BufferReader.asReader(buffer);
    return new NonExistentReadRequestHints(
      reader.readArray(readRequestLen, {
        fromBuffer: buf => NonMembershipHint.fromBuffer(buf, treeHeight, leafPreimageFromBuffer),
      }),
      reader.readNumbers(readRequestLen),
      reader.readArray(pendingValueLen, orderedValueFromBuffer),
      reader.readNumbers(pendingValueLen),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.nonMembershipHints, this.nextPendingValueIndices);
  }
}

export type NullifierNonExistentReadRequestHints = NonExistentReadRequestHints<
  typeof MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  typeof NULLIFIER_TREE_HEIGHT,
  IndexedTreeLeafPreimage,
  typeof MAX_NEW_NULLIFIERS_PER_TX,
  SideEffectLinkedToNoteHash
>;

export function nullifierNonExistentReadRequestHintsFromBuffer(
  buffer: Buffer | BufferReader,
): NullifierNonExistentReadRequestHints {
  return NonExistentReadRequestHints.fromBuffer(
    buffer,
    MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
    NULLIFIER_TREE_HEIGHT,
    NullifierLeafPreimage,
    MAX_NEW_NULLIFIERS_PER_TX,
    SideEffectLinkedToNoteHash,
  );
}

export class NullifierNonExistentReadRequestHintsBuilder {
  private hints: NullifierNonExistentReadRequestHints;
  private readRequestIndex = 0;

  constructor(
    sortedPendingNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    sortedPendingNullifierIndexHints: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {
    this.hints = new NonExistentReadRequestHints(
      makeTuple(MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, () =>
        NonMembershipHint.empty(NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage.empty),
      ),
      makeTuple(MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, () => 0),
      sortedPendingNullifiers,
      sortedPendingNullifierIndexHints,
    );
  }

  static empty() {
    const emptySortedPendingNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty);
    const emptySortedPendingNullifierIndexHints = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, () => 0);
    return new NullifierNonExistentReadRequestHintsBuilder(
      emptySortedPendingNullifiers,
      emptySortedPendingNullifierIndexHints,
    ).toHints();
  }

  addHint(
    membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>,
    lowLeafPreimage: IndexedTreeLeafPreimage,
    nextPendingValueIndex: number,
  ) {
    this.hints.nonMembershipHints[this.readRequestIndex] = new NonMembershipHint(membershipWitness, lowLeafPreimage);
    this.hints.nextPendingValueIndices[this.readRequestIndex] = nextPendingValueIndex;
    this.readRequestIndex++;
  }

  toHints() {
    return this.hints;
  }
}
