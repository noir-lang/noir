import {
  AztecAddress,
  Fr,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  NullifierReadRequestResetHints,
  NullifierReadRequestResetHintsBuilder,
  PendingReadHint,
  ReadRequestContext,
  ReadRequestState,
  ReadRequestStatus,
  SettledReadHint,
  SideEffectLinkedToNoteHash,
} from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/hash';
import { makeTuple } from '@aztec/foundation/array';
import { Tuple } from '@aztec/foundation/serialize';

import { HintsBuildingDataOracle, buildNullifierReadRequestResetHints } from './build_hints.js';

describe('buildNullifierReadRequestResetHints', () => {
  const contractAddress = AztecAddress.random();
  const settledNullifierInnerValue = 99999;
  const settledNullifierValue = makeNullifier(settledNullifierInnerValue).value;
  const oracle: HintsBuildingDataOracle = {
    getNullifierMembershipWitness: value =>
      value.equals(settledNullifierValue) ? ({ membershipWitness: {}, leafPreimage: {} } as any) : undefined,
  };
  let nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>;
  let nullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>;
  let expectedHints: NullifierReadRequestResetHints;
  let numReadRequests = 0;
  let numPendingReads = 0;
  let numSettledReads = 0;

  const innerNullifier = (index: number) => index + 1;

  const makeReadRequest = (value: number, counter = 2) =>
    new ReadRequestContext(new Fr(value), counter, contractAddress);

  function makeNullifier(value: number, counter = 1) {
    const siloedValue = siloNullifier(contractAddress, new Fr(value));
    return new SideEffectLinkedToNoteHash(siloedValue, new Fr(0), new Fr(counter));
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

  const buildHints = () => buildNullifierReadRequestResetHints(oracle, nullifierReadRequests, nullifiers);

  beforeEach(() => {
    nullifierReadRequests = makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ReadRequestContext.empty);
    nullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i => makeNullifier(innerNullifier(i)));
    expectedHints = NullifierReadRequestResetHintsBuilder.empty();
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

  it('throws if reading an unknown nullifier', async () => {
    nullifierReadRequests[0] = makeReadRequest(88888);
    await expect(buildHints()).rejects.toThrow('Read request is reading an unknown nullifier value.');
  });
});
