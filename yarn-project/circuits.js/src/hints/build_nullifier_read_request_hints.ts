import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { type Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';
import { type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

import {
  type MAX_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  type NULLIFIER_TREE_HEIGHT,
} from '../constants.gen.js';
import { siloNullifier } from '../hash/hash.js';
import {
  type MembershipWitness,
  Nullifier,
  NullifierReadRequestHintsBuilder,
  ReadRequest,
  type ScopedNullifier,
  ScopedReadRequest,
} from '../structs/index.js';
import { countAccumulatedItems, getNonEmptyItems } from '../utils/index.js';
import { ScopedValueCache } from './scoped_value_cache.js';

export function isValidNullifierReadRequest(readRequest: ScopedReadRequest, nullifier: ScopedNullifier) {
  return (
    readRequest.value.equals(nullifier.value) &&
    nullifier.contractAddress.equals(readRequest.contractAddress) &&
    readRequest.counter > nullifier.counter
  );
}

interface NullifierMembershipWitnessWithPreimage {
  membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>;
  leafPreimage: IndexedTreeLeafPreimage;
}

export async function buildNullifierReadRequestHints<PENDING extends number, SETTLED extends number>(
  oracle: {
    getNullifierMembershipWitness(nullifier: Fr): Promise<NullifierMembershipWitnessWithPreimage>;
  },
  nullifierReadRequests: Tuple<ScopedReadRequest, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<ScopedNullifier, typeof MAX_NULLIFIERS_PER_TX>,
  sizePending: PENDING,
  sizeSettled: SETTLED,
  futureNullifiers: ScopedNullifier[],
  siloed = false,
) {
  const builder = new NullifierReadRequestHintsBuilder(sizePending, sizeSettled);

  const numReadRequests = countAccumulatedItems(nullifierReadRequests);

  const nullifierMap: Map<bigint, { nullifier: ScopedNullifier; index: number }[]> = new Map();
  getNonEmptyItems(nullifiers).forEach((nullifier, index) => {
    const value = nullifier.value.toBigInt();
    const arr = nullifierMap.get(value) ?? [];
    arr.push({ nullifier, index });
    nullifierMap.set(value, arr);
  });

  const futureNullifiersMap = new ScopedValueCache(futureNullifiers);

  for (let i = 0; i < numReadRequests; ++i) {
    const readRequest = nullifierReadRequests[i];
    const pendingNullifier = nullifierMap
      .get(readRequest.value.toBigInt())
      ?.find(({ nullifier }) => isValidNullifierReadRequest(readRequest, nullifier));

    if (pendingNullifier !== undefined) {
      builder.addPendingReadRequest(i, pendingNullifier.index);
    } else if (
      !futureNullifiersMap
        .get(readRequest)
        .some(futureNullifier => isValidNullifierReadRequest(readRequest, futureNullifier))
    ) {
      const siloedValue = siloed ? readRequest.value : siloNullifier(readRequest.contractAddress, readRequest.value);
      const membershipWitnessWithPreimage = await oracle.getNullifierMembershipWitness(siloedValue);
      builder.addSettledReadRequest(
        i,
        membershipWitnessWithPreimage.membershipWitness,
        membershipWitnessWithPreimage.leafPreimage,
      );
    }
  }
  return builder.toHints();
}

export function buildSiloedNullifierReadRequestHints<PENDING extends number, SETTLED extends number>(
  oracle: {
    getNullifierMembershipWitness(nullifier: Fr): Promise<NullifierMembershipWitnessWithPreimage>;
  },
  nullifierReadRequests: Tuple<ScopedReadRequest, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<Nullifier, typeof MAX_NULLIFIERS_PER_TX>,
  sizePending: PENDING,
  sizeSettled: SETTLED,
) {
  // Nullifiers outputted from public kernels are already siloed while read requests are not.
  // Siloing the read request values and set the contract addresses to zero to find the matching nullifier contexts.
  const siloedReadRequests = padArrayEnd(
    getNonEmptyItems(nullifierReadRequests).map(r =>
      new ReadRequest(siloNullifier(r.contractAddress, r.value), r.counter).scope(AztecAddress.ZERO),
    ),
    ScopedReadRequest.empty(),
    MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  );

  const scopedNullifiers = nullifiers.map(n =>
    new Nullifier(n.value, n.counter, n.noteHash).scope(AztecAddress.ZERO),
  ) as Tuple<ScopedNullifier, typeof MAX_NULLIFIERS_PER_TX>;

  return buildNullifierReadRequestHints(
    oracle,
    siloedReadRequests,
    scopedNullifiers,
    sizePending,
    sizeSettled,
    [],
    true,
  );
}
