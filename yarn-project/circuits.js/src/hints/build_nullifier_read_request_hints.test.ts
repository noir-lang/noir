import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';

import { MAX_NEW_NULLIFIERS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX } from '../constants.gen.js';
import { siloNullifier } from '../hash/index.js';
import {
  Nullifier,
  type NullifierReadRequestHints,
  NullifierReadRequestHintsBuilder,
  PendingReadHint,
  ReadRequestContext,
  ReadRequestState,
  ReadRequestStatus,
  SettledReadHint,
} from '../structs/index.js';
import { buildNullifierReadRequestHints } from './build_nullifier_read_request_hints.js';

describe('buildNullifierReadRequestHints', () => {
  const contractAddress = AztecAddress.random();
  const settledNullifierInnerValue = 99999;
  const oracle = {
    getNullifierMembershipWitness: () => ({ membershipWitness: {}, leafPreimage: {} } as any),
  };
  let nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>;
  let nullifiers: Tuple<Nullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>;
  let expectedHints: NullifierReadRequestHints;
  let numReadRequests = 0;
  let numPendingReads = 0;
  let numSettledReads = 0;

  const innerNullifier = (index: number) => index + 1;

  const makeReadRequest = (value: number, counter = 2) =>
    new ReadRequestContext(new Fr(value), counter, contractAddress);

  function makeNullifier(value: number, counter = 1) {
    const siloedValue = siloNullifier(contractAddress, new Fr(value));
    return new Nullifier(siloedValue, counter, Fr.ZERO);
  }

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

  const buildHints = () => buildNullifierReadRequestHints(oracle, nullifierReadRequests, nullifiers);

  beforeEach(() => {
    nullifierReadRequests = makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ReadRequestContext.empty);
    nullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i => makeNullifier(innerNullifier(i)));
    expectedHints = NullifierReadRequestHintsBuilder.empty();
    numReadRequests = 0;
    numPendingReads = 0;
    numSettledReads = 0;
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

  it('builds hints for mixed pending and settled nullifier read requests', async () => {
    readPendingNullifier({ nullifierIndex: 2 });
    readSettledNullifier();
    readSettledNullifier();
    readPendingNullifier({ nullifierIndex: 1 });
    readPendingNullifier({ nullifierIndex: 1 });
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });
});
