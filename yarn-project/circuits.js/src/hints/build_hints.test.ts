import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';

import { MAX_NEW_NULLIFIERS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX } from '../constants.gen.js';
import { siloNullifier } from '../hash/index.js';
import {
  NullifierNonExistentReadRequestHintsBuilder,
  type NullifierReadRequestHints,
  NullifierReadRequestHintsBuilder,
  PendingReadHint,
  ReadRequestContext,
  ReadRequestState,
  ReadRequestStatus,
  SettledReadHint,
  SideEffectLinkedToNoteHash,
} from '../structs/index.js';
import { buildNullifierNonExistentReadRequestHints, buildNullifierReadRequestHints } from './build_hints.js';

describe('buildNullifierReadRequestHints', () => {
  const contractAddress = AztecAddress.random();
  const settledNullifierInnerValue = 99999;
  const settledNullifierValue = makeNullifier(settledNullifierInnerValue).value;
  const oracle = {
    getNullifierMembershipWitness: (value: Fr) =>
      value.equals(settledNullifierValue) ? ({ membershipWitness: {}, leafPreimage: {} } as any) : undefined,
  };
  let nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>;
  let nullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>;
  let expectedHints: NullifierReadRequestHints;
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

  it('throws if reading an unknown nullifier', async () => {
    nullifierReadRequests[0] = makeReadRequest(88888);
    await expect(buildHints()).rejects.toThrow('Read request is reading an unknown nullifier value.');
  });
});

describe('buildNullifierNonExistentReadRequestHints', () => {
  const contractAddress = AztecAddress.random();
  const oracle = {
    getLowNullifierMembershipWitness: () => ({ membershipWitness: {}, leafPreimage: {} } as any),
  };
  const nonExistentReadRequests = makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ReadRequestContext.empty);
  let nullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty);

  const innerNullifier = (index: number) => index + 1;

  const makeReadRequest = (value: number, counter = 2) =>
    new ReadRequestContext(new Fr(value), counter, contractAddress);

  const makeNullifier = (value: number, counter = 1) => {
    const siloedValue = siloNullifier(contractAddress, new Fr(value));
    return new SideEffectLinkedToNoteHash(siloedValue, new Fr(0), new Fr(counter));
  };

  interface TestNullifier {
    value: number;
    siloedValue: Fr;
  }

  const populateNullifiers = (numNullifiers = MAX_NEW_NULLIFIERS_PER_TX) => {
    nullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i =>
      i < numNullifiers ? makeNullifier(innerNullifier(i)) : SideEffectLinkedToNoteHash.empty(),
    );
  };

  const generateSortedNullifiers = (numNullifiers: number) => {
    const nullifiers: TestNullifier[] = [];
    for (let i = 0; i < numNullifiers; ++i) {
      const value = i;
      nullifiers.push({
        value,
        siloedValue: siloNullifier(contractAddress, new Fr(value)),
      });
    }
    return nullifiers.sort((a, b) => (b.siloedValue.lt(a.siloedValue) ? 1 : -1));
  };

  const buildHints = () => buildNullifierNonExistentReadRequestHints(oracle, nonExistentReadRequests, nullifiers);

  it('builds empty hints', async () => {
    const hints = await buildHints();
    const emptyHints = NullifierNonExistentReadRequestHintsBuilder.empty();
    expect(hints).toEqual(emptyHints);
  });

  it('builds hints for full sorted nullifiers', async () => {
    populateNullifiers();

    const hints = await buildHints();
    const { sortedPendingValues, sortedPendingValueHints } = hints;
    for (let i = 0; i < sortedPendingValues.length - 1; ++i) {
      expect(sortedPendingValues[i].value.lt(sortedPendingValues[i + 1].value)).toBe(true);
    }
    for (let i = 0; i < nullifiers.length; ++i) {
      const index = sortedPendingValueHints[i];
      expect(nullifiers[i].value.equals(sortedPendingValues[index].value)).toBe(true);
    }
  });

  it('builds hints for half-full sorted nullifiers', async () => {
    const numNonEmptyNullifiers = MAX_NEW_NULLIFIERS_PER_TX / 2;
    populateNullifiers(numNonEmptyNullifiers);

    const hints = await buildHints();
    const { sortedPendingValues, sortedPendingValueHints } = hints;

    // The first half contains sorted values.
    for (let i = 0; i < numNonEmptyNullifiers - 1; ++i) {
      expect(sortedPendingValues[i]).not.toEqual(SideEffectLinkedToNoteHash.empty());
      expect(sortedPendingValues[i].value.lt(sortedPendingValues[i + 1].value)).toBe(true);
    }
    for (let i = 0; i < numNonEmptyNullifiers; ++i) {
      const index = sortedPendingValueHints[i];
      expect(nullifiers[i].value.equals(sortedPendingValues[index].value)).toBe(true);
    }

    // The second half is empty.
    for (let i = numNonEmptyNullifiers; i < sortedPendingValues.length; ++i) {
      expect(sortedPendingValues[i]).toEqual(SideEffectLinkedToNoteHash.empty());
    }
    for (let i = numNonEmptyNullifiers; i < sortedPendingValueHints.length; ++i) {
      expect(sortedPendingValueHints[i]).toBe(0);
    }
  });

  it('builds hints for read requests', async () => {
    const numNonEmptyNullifiers = MAX_NEW_NULLIFIERS_PER_TX / 2;
    expect(numNonEmptyNullifiers > 1).toBe(true); // Need at least 2 nullifiers to test a value in the middle.

    const sortedNullifiers = generateSortedNullifiers(numNonEmptyNullifiers + 3);
    const minNullifier = sortedNullifiers.splice(0, 1)[0];
    const maxNullifier = sortedNullifiers.pop()!;
    const midIndex = Math.floor(numNonEmptyNullifiers / 2);
    const midNullifier = sortedNullifiers.splice(midIndex, 1)[0];

    nonExistentReadRequests[0] = makeReadRequest(midNullifier.value);
    nonExistentReadRequests[1] = makeReadRequest(maxNullifier.value);
    nonExistentReadRequests[2] = makeReadRequest(minNullifier.value);
    nullifiers = padArrayEnd(
      sortedNullifiers.map(n => makeNullifier(n.value)),
      SideEffectLinkedToNoteHash.empty(),
      MAX_NEW_NULLIFIERS_PER_TX,
    );

    const hints = await buildHints();
    const { nextPendingValueIndices } = hints;
    expect(nextPendingValueIndices.slice(0, 3)).toEqual([midIndex, numNonEmptyNullifiers, 0]);
  });

  it('throws if reading existing value', async () => {
    populateNullifiers();

    nonExistentReadRequests[0] = makeReadRequest(innerNullifier(2));

    await expect(() => buildHints()).rejects.toThrow('Nullifier exists in the pending set.');
  });
});
