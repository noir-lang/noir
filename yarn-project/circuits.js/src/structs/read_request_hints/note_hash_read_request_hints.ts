import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { type BufferReader } from '@aztec/foundation/serialize';

import { MAX_NOTE_HASH_READ_REQUESTS_PER_TX, NOTE_HASH_TREE_HEIGHT } from '../../constants.gen.js';
import { type MembershipWitness } from '../membership_witness.js';
import {
  PendingReadHint,
  ReadRequestResetHints,
  ReadRequestState,
  ReadRequestStatus,
  SettledReadHint,
} from './read_request_hints.js';

type NoteHashLeafValue = Fr;

export type NoteHashReadRequestHints<PENDING extends number, SETTLED extends number> = ReadRequestResetHints<
  typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  PENDING,
  SETTLED,
  typeof NOTE_HASH_TREE_HEIGHT,
  NoteHashLeafValue
>;

export function noteHashReadRequestHintsFromBuffer<PENDING extends number, SETTLED extends number>(
  buffer: Buffer | BufferReader,
  numPending: PENDING,
  numSettled: SETTLED,
): NoteHashReadRequestHints<PENDING, SETTLED> {
  return ReadRequestResetHints.fromBuffer(
    buffer,
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    numPending,
    numSettled,
    NOTE_HASH_TREE_HEIGHT,
    Fr,
  );
}

export class NoteHashReadRequestHintsBuilder<PENDING extends number, SETTLED extends number> {
  private hints: NoteHashReadRequestHints<PENDING, SETTLED>;
  public numPendingReadHints = 0;
  public numSettledReadHints = 0;

  constructor(numPending: PENDING, numSettled: SETTLED) {
    this.hints = new ReadRequestResetHints(
      makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ReadRequestStatus.nada),
      makeTuple(numPending, () => PendingReadHint.nada(MAX_NOTE_HASH_READ_REQUESTS_PER_TX)),
      makeTuple(numSettled, () =>
        SettledReadHint.nada(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, NOTE_HASH_TREE_HEIGHT, Fr.zero),
      ),
    );
  }

  static empty<PENDING extends number, SETTLED extends number>(numPending: PENDING, numSettled: SETTLED) {
    return new NoteHashReadRequestHintsBuilder(numPending, numSettled).toHints().hints;
  }

  addPendingReadRequest(readRequestIndex: number, noteHashIndex: number) {
    this.hints.readRequestStatuses[readRequestIndex] = new ReadRequestStatus(
      ReadRequestState.PENDING,
      this.numPendingReadHints,
    );
    this.hints.pendingReadHints[this.numPendingReadHints] = new PendingReadHint(readRequestIndex, noteHashIndex);
    this.numPendingReadHints++;
  }

  addSettledReadRequest(
    readRequestIndex: number,
    membershipWitness: MembershipWitness<typeof NOTE_HASH_TREE_HEIGHT>,
    value: NoteHashLeafValue,
  ) {
    this.hints.readRequestStatuses[readRequestIndex] = new ReadRequestStatus(
      ReadRequestState.SETTLED,
      this.numSettledReadHints,
    );
    this.hints.settledReadHints[this.numSettledReadHints] = new SettledReadHint(
      readRequestIndex,
      membershipWitness,
      value,
    );
    this.numSettledReadHints++;
  }

  toHints() {
    return {
      numPendingReadHints: this.numPendingReadHints,
      numSettledReadHints: this.numSettledReadHints,
      hints: this.hints,
    };
  }
}
