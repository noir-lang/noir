import { padArrayEnd } from '@aztec/foundation/collection';
import { type Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';
import { type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

import {
  MAX_NEW_NULLIFIERS_PER_TX,
  type MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  type MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  type NULLIFIER_TREE_HEIGHT,
} from '../constants.gen.js';
import { siloNullifier } from '../hash/index.js';
import { type MembershipWitness } from '../structs/membership_witness.js';
import { NullifierNonExistentReadRequestHintsBuilder } from '../structs/non_existent_read_request_hints.js';
import { type ReadRequestContext } from '../structs/read_request.js';
import { NullifierReadRequestHintsBuilder } from '../structs/read_request_hints.js';
import { SideEffectLinkedToNoteHash } from '../structs/side_effects.js';
import { countAccumulatedItems } from '../utils/index.js';

export interface NullifierMembershipWitnessWithPreimage {
  membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>;
  leafPreimage: IndexedTreeLeafPreimage;
}

export async function buildNullifierReadRequestHints(
  oracle: {
    getNullifierMembershipWitness(nullifier: Fr): Promise<NullifierMembershipWitnessWithPreimage | undefined>;
  },
  nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
) {
  const builder = new NullifierReadRequestHintsBuilder();

  const numReadRequests = countAccumulatedItems(nullifierReadRequests);

  const nullifierIndexMap: Map<bigint, number> = new Map();
  nullifiers.forEach((n, i) => nullifierIndexMap.set(n.value.toBigInt(), i));

  for (let i = 0; i < numReadRequests; ++i) {
    const readRequest = nullifierReadRequests[i];
    // TODO - Should be comparing un-siloed values and contract addresses.
    const value = siloNullifier(readRequest.contractAddress, readRequest.value);

    const pendingValueIndex = nullifierIndexMap.get(value.toBigInt());
    if (pendingValueIndex !== undefined) {
      builder.addPendingReadRequest(i, pendingValueIndex);
    } else {
      const membershipWitnessWithPreimage = await oracle.getNullifierMembershipWitness(value);
      if (!membershipWitnessWithPreimage) {
        throw new Error('Read request is reading an unknown nullifier value.');
      }

      builder.addSettledReadRequest(
        i,
        membershipWitnessWithPreimage.membershipWitness,
        membershipWitnessWithPreimage.leafPreimage,
      );
    }
  }
  return builder.toHints();
}

interface SortedResult<T, N extends number> {
  sortedValues: Tuple<T, N>;
  sortedIndexHints: Tuple<number, N>;
}

function sortNullifiersByValues(
  nullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
): SortedResult<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX> {
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
      SideEffectLinkedToNoteHash.empty(),
      MAX_NEW_NULLIFIERS_PER_TX,
    ),
    sortedIndexHints: padArrayEnd(sortedIndexHints, 0, MAX_NEW_NULLIFIERS_PER_TX),
  };
}

export async function buildNullifierNonExistentReadRequestHints(
  oracle: {
    getLowNullifierMembershipWitness(nullifier: Fr): Promise<NullifierMembershipWitnessWithPreimage>;
  },
  nullifierNonExistentReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
  pendingNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
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
    } else if (sortedValues[nextPendingValueIndex].value.equals(siloedValue)) {
      throw new Error('Nullifier exists in the pending set.');
    }

    builder.addHint(membershipWitness, leafPreimage, nextPendingValueIndex);
  }

  return builder.toHints();
}
