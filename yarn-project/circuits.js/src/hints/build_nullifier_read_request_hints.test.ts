import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';

import { MAX_NULLIFIERS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX } from '../constants.gen.js';
import {
  Nullifier,
  type NullifierReadRequestHints,
  NullifierReadRequestHintsBuilder,
  PendingReadHint,
  ReadRequest,
  ReadRequestState,
  ReadRequestStatus,
  type ScopedNullifier,
  ScopedReadRequest,
  SettledReadHint,
} from '../structs/index.js';
import { buildNullifierReadRequestHints } from './build_nullifier_read_request_hints.js';

describe('buildNullifierReadRequestHints', () => {
  const contractAddress = AztecAddress.random();
  const settledNullifierInnerValue = 99999;
  const oracle = {
    getNullifierMembershipWitness: () => ({ membershipWitness: {}, leafPreimage: {} } as any),
  };
  let nullifierReadRequests: Tuple<ScopedReadRequest, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>;
  let nullifiers: Tuple<ScopedNullifier, typeof MAX_NULLIFIERS_PER_TX>;
  let expectedHints: NullifierReadRequestHints<
    typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX
  >;
  let numReadRequests = 0;
  let numPendingReads = 0;
  let numSettledReads = 0;
  let futureNullifiers: ScopedNullifier[];

  const innerNullifier = (index: number) => index + 1;

  const makeReadRequest = (value: number, counter = 2) =>
    new ReadRequest(new Fr(value), counter).scope(contractAddress);

  const makeNullifier = (value: number, counter = 1) =>
    new Nullifier(new Fr(value), counter, Fr.ZERO).scope(contractAddress);

  const readPendingNullifier = ({
    nullifierIndex,
    readRequestIndex = numReadRequests,
    hintIndex = numPendingReads,
  }: {
    nullifierIndex: number;
    readRequestIndex?: number;
    hintIndex?: number;
  }) => {
    nullifierReadRequests[readRequestIndex] = makeReadRequest(innerNullifier(nullifierIndex));
    expectedHints.readRequestStatuses[readRequestIndex] = new ReadRequestStatus(ReadRequestState.PENDING, hintIndex);
    expectedHints.pendingReadHints[hintIndex] = new PendingReadHint(readRequestIndex, nullifierIndex);
    numReadRequests++;
    numPendingReads++;
  };

  const readSettledNullifier = ({
    readRequestIndex = numReadRequests,
    hintIndex = numSettledReads,
  }: {
    readRequestIndex?: number;
    hintIndex?: number;
  } = {}) => {
    nullifierReadRequests[readRequestIndex] = makeReadRequest(settledNullifierInnerValue);
    expectedHints.readRequestStatuses[readRequestIndex] = new ReadRequestStatus(ReadRequestState.SETTLED, hintIndex);
    expectedHints.settledReadHints[hintIndex] = new SettledReadHint(readRequestIndex, {} as any, {} as any);
    numReadRequests++;
    numSettledReads++;
  };

  const readFutureNullifier = (nullifierIndex: number) => {
    const readRequestIndex = numReadRequests;
    nullifierReadRequests[readRequestIndex] = makeReadRequest(futureNullifiers[nullifierIndex].value.toNumber());
    numReadRequests++;
  };

  const buildHints = async () =>
    (
      await buildNullifierReadRequestHints(
        oracle,
        nullifierReadRequests,
        nullifiers,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX,
        futureNullifiers,
      )
    ).hints;

  beforeEach(() => {
    nullifierReadRequests = makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest.empty);
    nullifiers = makeTuple(MAX_NULLIFIERS_PER_TX, i => makeNullifier(innerNullifier(i)));
    expectedHints = NullifierReadRequestHintsBuilder.empty(
      MAX_NULLIFIER_READ_REQUESTS_PER_TX,
      MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    );
    numReadRequests = 0;
    numPendingReads = 0;
    numSettledReads = 0;
    futureNullifiers = new Array(MAX_NULLIFIERS_PER_TX)
      .fill(null)
      .map((_, i) => makeNullifier(innerNullifier(i + MAX_NULLIFIERS_PER_TX)));
  });

  it('builds empty hints', async () => {
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for pending nullifier read requests', async () => {
    readPendingNullifier({ nullifierIndex: 2 });
    readPendingNullifier({ nullifierIndex: 1 });
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for settled nullifier read requests', async () => {
    readSettledNullifier();
    readSettledNullifier();
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for mixed pending and settled and future nullifier read requests', async () => {
    readPendingNullifier({ nullifierIndex: 2 });
    readSettledNullifier();
    readFutureNullifier(0);
    readSettledNullifier();
    readPendingNullifier({ nullifierIndex: 1 });
    readFutureNullifier(1);
    readPendingNullifier({ nullifierIndex: 1 });
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });
});
