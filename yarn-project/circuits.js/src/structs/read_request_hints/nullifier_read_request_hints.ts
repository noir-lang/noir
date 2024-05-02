import { makeTuple } from '@aztec/foundation/array';
import { type BufferReader } from '@aztec/foundation/serialize';
import { type TreeLeafPreimage } from '@aztec/foundation/trees';

import { MAX_NULLIFIER_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT } from '../../constants.gen.js';
import { type MembershipWitness } from '../membership_witness.js';
import { NullifierLeafPreimage } from '../trees/index.js';
import {
  PendingReadHint,
  ReadRequestResetHints,
  ReadRequestState,
  ReadRequestStatus,
  SettledReadHint,
} from './read_request_hints.js';

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
