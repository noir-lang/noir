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

export type NullifierReadRequestHints<PENDING extends number, SETTLED extends number> = ReadRequestResetHints<
  typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  PENDING,
  SETTLED,
  typeof NULLIFIER_TREE_HEIGHT,
  TreeLeafPreimage
>;

export function nullifierReadRequestHintsFromBuffer<PENDING extends number, SETTLED extends number>(
  buffer: Buffer | BufferReader,
  numPendingReads: PENDING,
  numSettledReads: SETTLED,
): NullifierReadRequestHints<PENDING, SETTLED> {
  return ReadRequestResetHints.fromBuffer(
    buffer,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    numPendingReads,
    numSettledReads,
    NULLIFIER_TREE_HEIGHT,
    NullifierLeafPreimage,
  );
}

export class NullifierReadRequestHintsBuilder<PENDING extends number, SETTLED extends number> {
  private hints: NullifierReadRequestHints<PENDING, SETTLED>;
  private numPendingReadHints = 0;
  private numSettledReadHints = 0;

  constructor(numPending: PENDING, numSettled: SETTLED) {
    this.hints = new ReadRequestResetHints(
      makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ReadRequestStatus.nada),
      makeTuple(numPending, () => PendingReadHint.nada(MAX_NULLIFIER_READ_REQUESTS_PER_TX)),
      makeTuple(numSettled, () =>
        SettledReadHint.nada(MAX_NULLIFIER_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage.empty),
      ),
    );
  }

  static empty<PENDING extends number, SETTLED extends number>(numPending: PENDING, numSettled: SETTLED) {
    return new NullifierReadRequestHintsBuilder(numPending, numSettled).toHints().hints;
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
    return {
      numPendingReadHints: this.numPendingReadHints,
      numSettledReadHints: this.numSettledReadHints,
      hints: this.hints,
    };
  }
}
