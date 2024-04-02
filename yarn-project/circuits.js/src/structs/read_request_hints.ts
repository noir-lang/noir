import { makeTuple } from '@aztec/foundation/array';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { type TreeLeafPreimage } from '@aztec/foundation/trees';

import { MAX_NULLIFIER_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT } from '../constants.gen.js';
import { MembershipWitness } from './membership_witness.js';
import { NullifierLeafPreimage } from './rollup/nullifier_leaf/index.js';

export enum ReadRequestState {
  NADA = 0,
  PENDING = 1,
  SETTLED = 2,
}

export class ReadRequestStatus {
  constructor(public state: ReadRequestState, public hintIndex: number) {}

  static nada() {
    return new ReadRequestStatus(ReadRequestState.NADA, 0);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ReadRequestStatus(reader.readNumber(), reader.readNumber());
  }

  toBuffer() {
    return serializeToBuffer(this.state, this.hintIndex);
  }
}

export class PendingReadHint {
  constructor(public readRequestIndex: number, public pendingValueIndex: number) {}

  static nada(readRequestLen: number) {
    return new PendingReadHint(readRequestLen, 0);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PendingReadHint(reader.readNumber(), reader.readNumber());
  }

  toBuffer() {
    return serializeToBuffer(this.readRequestIndex, this.pendingValueIndex);
  }
}

export class SettledReadHint<TREE_HEIGHT extends number, LEAF_PREIMAGE extends TreeLeafPreimage> {
  constructor(
    public readRequestIndex: number,
    public membershipWitness: MembershipWitness<TREE_HEIGHT>,
    public leafPreimage: LEAF_PREIMAGE,
  ) {}

  static nada<TREE_HEIGHT extends number, LEAF_PREIMAGE extends TreeLeafPreimage>(
    readRequestLen: number,
    treeHeight: TREE_HEIGHT,
    emptyLeafPreimage: () => LEAF_PREIMAGE,
  ) {
    return new SettledReadHint(readRequestLen, MembershipWitness.empty(treeHeight, 0n), emptyLeafPreimage());
  }

  static fromBuffer<TREE_HEIGHT extends number, LEAF_PREIMAGE extends TreeLeafPreimage>(
    buffer: Buffer | BufferReader,
    treeHeight: TREE_HEIGHT,
    leafPreimage: { fromBuffer(buffer: BufferReader): LEAF_PREIMAGE },
  ): SettledReadHint<TREE_HEIGHT, LEAF_PREIMAGE> {
    const reader = BufferReader.asReader(buffer);
    return new SettledReadHint(
      reader.readNumber(),
      MembershipWitness.fromBuffer(reader, treeHeight),
      reader.readObject(leafPreimage),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.readRequestIndex, this.membershipWitness, this.leafPreimage);
  }
}

/**
 * Hints for read request reset circuit.
 */
export class ReadRequestResetHints<
  READ_REQUEST_LEN extends number,
  NUM_PENDING_READS extends number,
  NUM_SETTLED_READS extends number,
  TREE_HEIGHT extends number,
  LEAF_PREIMAGE extends TreeLeafPreimage,
> {
  constructor(
    public readRequestStatuses: Tuple<ReadRequestStatus, READ_REQUEST_LEN>,
    /**
     * The hints for read requests reading pending values.
     */
    public pendingReadHints: Tuple<PendingReadHint, NUM_PENDING_READS>,
    /**
     * The hints for read requests reading settled values.
     */
    public settledReadHints: Tuple<SettledReadHint<TREE_HEIGHT, LEAF_PREIMAGE>, NUM_SETTLED_READS>,
  ) {}

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer<
    READ_REQUEST_LEN extends number,
    NUM_PENDING_READS extends number,
    NUM_SETTLED_READS extends number,
    TREE_HEIGHT extends number,
    LEAF_PREIMAGE extends TreeLeafPreimage,
  >(
    buffer: Buffer | BufferReader,
    readRequestLen: READ_REQUEST_LEN,
    numPendingReads: NUM_PENDING_READS,
    numSettledReads: NUM_SETTLED_READS,
    treeHeight: TREE_HEIGHT,
    leafPreimageFromBuffer: { fromBuffer: (buffer: BufferReader) => LEAF_PREIMAGE },
  ): ReadRequestResetHints<READ_REQUEST_LEN, NUM_PENDING_READS, NUM_SETTLED_READS, TREE_HEIGHT, LEAF_PREIMAGE> {
    const reader = BufferReader.asReader(buffer);
    return new ReadRequestResetHints(
      reader.readArray(readRequestLen, ReadRequestStatus),
      reader.readArray(numPendingReads, PendingReadHint),
      reader.readArray(numSettledReads, {
        fromBuffer: r => SettledReadHint.fromBuffer(r, treeHeight, leafPreimageFromBuffer),
      }),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.readRequestStatuses, this.pendingReadHints, this.settledReadHints);
  }
}

export type NullifierReadRequestHints = ReadRequestResetHints<
  typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  typeof NULLIFIER_TREE_HEIGHT,
  TreeLeafPreimage
>;

export function nullifierReadRequestHintsFromBuffer(buffer: Buffer | BufferReader): NullifierReadRequestHints {
  return ReadRequestResetHints.fromBuffer(
    buffer,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    NULLIFIER_TREE_HEIGHT,
    NullifierLeafPreimage,
  );
}

export class NullifierReadRequestHintsBuilder {
  private hints: NullifierReadRequestHints;
  private numPendingReadHints = 0;
  private numSettledReadHints = 0;

  constructor() {
    this.hints = new ReadRequestResetHints(
      makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ReadRequestStatus.nada),
      makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, () => PendingReadHint.nada(MAX_NULLIFIER_READ_REQUESTS_PER_TX)),
      makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, () =>
        SettledReadHint.nada(MAX_NULLIFIER_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage.empty),
      ),
    );
  }

  static empty() {
    return new NullifierReadRequestHintsBuilder().toHints();
  }

  addPendingReadRequest(readRequestIndex: number, nullifierIndex: number) {
    this.hints.readRequestStatuses[readRequestIndex] = new ReadRequestStatus(
      ReadRequestState.PENDING,
      this.numPendingReadHints,
    );
    this.hints.pendingReadHints[this.numPendingReadHints] = new PendingReadHint(readRequestIndex, nullifierIndex);
    this.numPendingReadHints++;
  }

  addSettledReadRequest(
    readRequestIndex: number,
    membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>,
    leafPreimage: TreeLeafPreimage,
  ) {
    this.hints.readRequestStatuses[readRequestIndex] = new ReadRequestStatus(
      ReadRequestState.SETTLED,
      this.numSettledReadHints,
    );
    this.hints.settledReadHints[this.numSettledReadHints] = new SettledReadHint(
      readRequestIndex,
      membershipWitness,
      leafPreimage,
    );
    this.numSettledReadHints++;
  }

  toHints() {
    return this.hints;
  }
}
