import { type Tuple } from '@aztec/foundation/serialize';

import {
  type MAX_NOTE_HASHES_PER_TX,
  type MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  type NOTE_HASH_TREE_HEIGHT,
} from '../constants.gen.js';
import { siloNoteHash } from '../hash/index.js';
import {
  type MembershipWitness,
  NoteHashReadRequestHintsBuilder,
  type ScopedNoteHash,
  type ScopedReadRequest,
} from '../structs/index.js';
import { countAccumulatedItems, getNonEmptyItems } from '../utils/index.js';
import { ScopedValueCache } from './scoped_value_cache.js';

export function isValidNoteHashReadRequest(readRequest: ScopedReadRequest, noteHash: ScopedNoteHash) {
  return (
    noteHash.value.equals(readRequest.value) &&
    noteHash.contractAddress.equals(readRequest.contractAddress) &&
    readRequest.counter > noteHash.counter
  );
}

export async function buildNoteHashReadRequestHints<PENDING extends number, SETTLED extends number>(
  oracle: {
    getNoteHashMembershipWitness(leafIndex: bigint): Promise<MembershipWitness<typeof NOTE_HASH_TREE_HEIGHT>>;
  },
  noteHashReadRequests: Tuple<ScopedReadRequest, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
  noteHashes: Tuple<ScopedNoteHash, typeof MAX_NOTE_HASHES_PER_TX>,
  noteHashLeafIndexMap: Map<bigint, bigint>,
  sizePending: PENDING,
  sizeSettled: SETTLED,
  futureNoteHashes: ScopedNoteHash[],
) {
  const builder = new NoteHashReadRequestHintsBuilder(sizePending, sizeSettled);

  const numReadRequests = countAccumulatedItems(noteHashReadRequests);

  const noteHashMap: Map<bigint, { noteHash: ScopedNoteHash; index: number }[]> = new Map();
  getNonEmptyItems(noteHashes).forEach((noteHash, index) => {
    const value = noteHash.value.toBigInt();
    const arr = noteHashMap.get(value) ?? [];
    arr.push({ noteHash, index });
    noteHashMap.set(value, arr);
  });

  const futureNoteHashMap = new ScopedValueCache(futureNoteHashes);

  for (let i = 0; i < numReadRequests; ++i) {
    const readRequest = noteHashReadRequests[i];

    const value = readRequest.value;

    const pendingNoteHash = noteHashMap
      .get(value.toBigInt())
      ?.find(n => isValidNoteHashReadRequest(readRequest, n.noteHash));

    if (pendingNoteHash !== undefined) {
      builder.addPendingReadRequest(i, pendingNoteHash.index);
    } else if (
      !futureNoteHashMap
        .get(readRequest)
        .find(futureNoteHash => isValidNoteHashReadRequest(readRequest, futureNoteHash))
    ) {
      const siloedValue = siloNoteHash(readRequest.contractAddress, value);
      const leafIndex = noteHashLeafIndexMap.get(siloedValue.toBigInt());
      if (leafIndex === undefined) {
        throw new Error('Read request is reading an unknown note hash.');
      }
      const membershipWitness = await oracle.getNoteHashMembershipWitness(leafIndex);
      builder.addSettledReadRequest(i, membershipWitness, siloedValue);
    }
  }
  return builder.toHints();
}
