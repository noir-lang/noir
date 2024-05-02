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

export type NoteHashReadRequestHints = ReadRequestResetHints<
  typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  typeof NOTE_HASH_TREE_HEIGHT,
  NoteHashLeafValue
>;

export function noteHashReadRequestHintsFromBuffer(buffer: Buffer | BufferReader): NoteHashReadRequestHints {
  return ReadRequestResetHints.fromBuffer(
    buffer,
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    NOTE_HASH_TREE_HEIGHT,
    Fr,
  );
}

export class NoteHashReadRequestHintsBuilder {
  private hints: NoteHashReadRequestHints;
  private numPendingReadHints = 0;
  private numSettledReadHints = 0;

  constructor() {
    this.hints = new ReadRequestResetHints(
      makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ReadRequestStatus.nada),
      makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, () => PendingReadHint.nada(MAX_NOTE_HASH_READ_REQUESTS_PER_TX)),
      makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, () =>
        SettledReadHint.nada(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, NOTE_HASH_TREE_HEIGHT, Fr.zero),
      ),
    );
  }

  static empty() {
    return new NoteHashReadRequestHintsBuilder().toHints();
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
    return this.hints;
  }
}
