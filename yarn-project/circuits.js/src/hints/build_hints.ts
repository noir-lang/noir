import { Fr } from '@aztec/foundation/fields';
import { Tuple } from '@aztec/foundation/serialize';

import {
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  NULLIFIER_TREE_HEIGHT,
} from '../constants.gen.js';
import { siloNullifier } from '../hash/index.js';
import { MembershipWitness } from '../structs/membership_witness.js';
import { ReadRequestContext } from '../structs/read_request.js';
import { NullifierReadRequestResetHintsBuilder } from '../structs/read_request_reset_hints.js';
import { NullifierLeafPreimage } from '../structs/rollup/nullifier_leaf/index.js';
import { SideEffectLinkedToNoteHash } from '../structs/side_effects.js';
import { countAccumulatedItems } from './utils.js';

export interface NullifierMembershipWitnessWithPreimage {
  membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>;
  leafPreimage: NullifierLeafPreimage;
}

export interface HintsBuildingDataOracle {
  getNullifierMembershipWitness(nullifier: Fr): Promise<NullifierMembershipWitnessWithPreimage | undefined>;
}

export async function buildNullifierReadRequestResetHints(
  oracle: HintsBuildingDataOracle,
  nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
) {
  const builder = new NullifierReadRequestResetHintsBuilder();

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
