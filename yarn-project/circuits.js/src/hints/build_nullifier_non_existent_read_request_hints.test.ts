import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';

import { MAX_NULLIFIERS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX } from '../constants.gen.js';
import { siloNullifier } from '../hash/index.js';
import {
  Nullifier,
  NullifierNonExistentReadRequestHintsBuilder,
  ReadRequest,
  ScopedReadRequest,
} from '../structs/index.js';
import { buildNullifierNonExistentReadRequestHints } from './build_nullifier_non_existent_read_request_hints.js';

describe('buildNullifierNonExistentReadRequestHints', () => {
  const contractAddress = AztecAddress.random();
  const oracle = {
    getLowNullifierMembershipWitness: () => ({ membershipWitness: {}, leafPreimage: {} } as any),
  };
  const nonExistentReadRequests = makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest.empty);
  let nullifiers = makeTuple(MAX_NULLIFIERS_PER_TX, Nullifier.empty);

  const innerNullifier = (index: number) => index + 1;

  const makeReadRequest = (value: number, counter = 2) =>
    new ReadRequest(new Fr(value), counter).scope(contractAddress);

  const makeNullifier = (value: number, counter = 1) => {
    const siloedValue = siloNullifier(contractAddress, new Fr(value));
    return new Nullifier(siloedValue, 0, new Fr(counter));
  };

  interface TestNullifier {
    value: number;
    siloedValue: Fr;
  }

  const populateNullifiers = (numNullifiers = MAX_NULLIFIERS_PER_TX) => {
    nullifiers = makeTuple(MAX_NULLIFIERS_PER_TX, i =>
      i < numNullifiers ? makeNullifier(innerNullifier(i)) : Nullifier.empty(),
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
    const numNonEmptyNullifiers = MAX_NULLIFIERS_PER_TX / 2;
    populateNullifiers(numNonEmptyNullifiers);

    const hints = await buildHints();
    const { sortedPendingValues, sortedPendingValueHints } = hints;

    // The first half contains sorted values.
    for (let i = 0; i < numNonEmptyNullifiers - 1; ++i) {
      expect(sortedPendingValues[i]).not.toEqual(Nullifier.empty());
      expect(sortedPendingValues[i].value.lt(sortedPendingValues[i + 1].value)).toBe(true);
    }
    for (let i = 0; i < numNonEmptyNullifiers; ++i) {
      const index = sortedPendingValueHints[i];
      expect(nullifiers[i].value.equals(sortedPendingValues[index].value)).toBe(true);
    }

    // The second half is empty.
    for (let i = numNonEmptyNullifiers; i < sortedPendingValues.length; ++i) {
      expect(sortedPendingValues[i]).toEqual(Nullifier.empty());
    }
    for (let i = numNonEmptyNullifiers; i < sortedPendingValueHints.length; ++i) {
      expect(sortedPendingValueHints[i]).toBe(0);
    }
  });

  it('builds hints for read requests', async () => {
    const numNonEmptyNullifiers = MAX_NULLIFIERS_PER_TX / 2;
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
      Nullifier.empty(),
      MAX_NULLIFIERS_PER_TX,
    );

    const hints = await buildHints();
    const { nextPendingValueIndices } = hints;
    expect(nextPendingValueIndices.slice(0, 3)).toEqual([midIndex, numNonEmptyNullifiers, 0]);
  });

  it('throws if reading existing value', async () => {
    populateNullifiers();

    nonExistentReadRequests[0] = makeReadRequest(innerNullifier(2));

    await expect(() => buildHints()).rejects.toThrow(
      'Nullifier DOES exists in the pending set at the time of reading, but there is a NonExistentReadRequest for it.',
    );
  });
});
