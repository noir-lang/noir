import { padArrayEnd } from '@aztec/foundation/collection';
import { type Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';
import { type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

import {
  MAX_NULLIFIERS_PER_TX,
  type MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  type NULLIFIER_TREE_HEIGHT,
} from '../constants.gen.js';
import { siloNullifier } from '../hash/index.js';
import { Nullifier } from '../structs/index.js';
import { type MembershipWitness } from '../structs/membership_witness.js';
import { NullifierNonExistentReadRequestHintsBuilder } from '../structs/non_existent_read_request_hints.js';
import { type ScopedReadRequest } from '../structs/read_request.js';
import { countAccumulatedItems } from '../utils/index.js';

interface NullifierMembershipWitnessWithPreimage {
  membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>;
  leafPreimage: IndexedTreeLeafPreimage;
}

interface SortedResult<T, N extends number> {
  sortedValues: Tuple<T, N>;
  sortedIndexHints: Tuple<number, N>;
}

function sortNullifiersByValues(
  nullifiers: Tuple<Nullifier, typeof MAX_NULLIFIERS_PER_TX>,
): SortedResult<Nullifier, typeof MAX_NULLIFIERS_PER_TX> {
  const numNullifiers = countAccumulatedItems(nullifiers);
  const sorted = nullifiers
    .slice(0, numNullifiers)
    .map((nullifier, originalIndex) => ({ nullifier, originalIndex }))
    .sort((a, b) => (b.nullifier.value.lt(a.nullifier.value) ? 1 : -1));

  const sortedIndexHints: number[] = [];
  for (let i = 0; i < numNullifiers; ++i) {
    sortedIndexHints[sorted[i].originalIndex] = i;
  }

  return {
    sortedValues: padArrayEnd(
      sorted.map(s => s.nullifier),
      Nullifier.empty(),
      MAX_NULLIFIERS_PER_TX,
    ),
    sortedIndexHints: padArrayEnd(sortedIndexHints, 0, MAX_NULLIFIERS_PER_TX),
  };
}

export async function buildNullifierNonExistentReadRequestHints(
  oracle: {
    getLowNullifierMembershipWitness(nullifier: Fr): Promise<NullifierMembershipWitnessWithPreimage>;
  },
  nullifierNonExistentReadRequests: Tuple<ScopedReadRequest, typeof MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
  pendingNullifiers: Tuple<Nullifier, typeof MAX_NULLIFIERS_PER_TX>,
) {
  const { sortedValues, sortedIndexHints } = sortNullifiersByValues(pendingNullifiers);

  const builder = new NullifierNonExistentReadRequestHintsBuilder(sortedValues, sortedIndexHints);

  const numPendingNullifiers = countAccumulatedItems(pendingNullifiers);
  const numReadRequests = countAccumulatedItems(nullifierNonExistentReadRequests);
  for (let i = 0; i < numReadRequests; ++i) {
    const readRequest = nullifierNonExistentReadRequests[i];
    const siloedValue = siloNullifier(readRequest.contractAddress, readRequest.value);

    const { membershipWitness, leafPreimage } = await oracle.getLowNullifierMembershipWitness(siloedValue);

    let nextPendingValueIndex = sortedValues.findIndex(v => !v.value.lt(siloedValue));
    if (nextPendingValueIndex == -1) {
      nextPendingValueIndex = numPendingNullifiers;
    } else if (
      sortedValues[nextPendingValueIndex].value.equals(siloedValue) &&
      sortedValues[nextPendingValueIndex].counter < readRequest.counter
    ) {
      throw new Error(
        'Nullifier DOES exists in the pending set at the time of reading, but there is a NonExistentReadRequest for it.',
      );
    }

    builder.addHint(membershipWitness, leafPreimage, nextPendingValueIndex);
  }

  return builder.toHints();
}
