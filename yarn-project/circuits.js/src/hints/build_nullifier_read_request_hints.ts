import { type Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';
import { type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

import {
  type MAX_NEW_NULLIFIERS_PER_TX,
  type MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  type NULLIFIER_TREE_HEIGHT,
} from '../constants.gen.js';
import { siloNullifier } from '../hash/index.js';
import {
  type MembershipWitness,
  type Nullifier,
  NullifierReadRequestHintsBuilder,
  type ReadRequestContext,
} from '../structs/index.js';
import { countAccumulatedItems } from '../utils/index.js';

interface NullifierMembershipWitnessWithPreimage {
  membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>;
  leafPreimage: IndexedTreeLeafPreimage;
}

export async function buildNullifierReadRequestHints(
  oracle: {
    getNullifierMembershipWitness(nullifier: Fr): Promise<NullifierMembershipWitnessWithPreimage>;
  },
  nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<Nullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
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
      builder.addSettledReadRequest(
        i,
        membershipWitnessWithPreimage.membershipWitness,
        membershipWitnessWithPreimage.leafPreimage,
      );
    }
  }
  return builder.toHints();
}
