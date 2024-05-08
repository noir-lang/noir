import { type Tuple } from '@aztec/foundation/serialize';

import {
  type MAX_NEW_NOTE_HASHES_PER_TX,
  type MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  type NOTE_HASH_TREE_HEIGHT,
} from '../constants.gen.js';
import {
  type MembershipWitness,
  NoteHashReadRequestHintsBuilder,
  type ScopedNoteHash,
  type ScopedReadRequest,
} from '../structs/index.js';
import { countAccumulatedItems, getNonEmptyItems } from '../utils/index.js';

function isValidNoteHashReadRequest(readRequest: ScopedReadRequest, noteHash: ScopedNoteHash) {
  return (
    noteHash.contractAddress.equals(readRequest.contractAddress) &&
    readRequest.counter > noteHash.counter &&
    (noteHash.nullifierCounter === 0 || noteHash.nullifierCounter > readRequest.counter)
  );
}

export async function buildNoteHashReadRequestHints(
  oracle: {
    getNoteHashMembershipWitness(leafIndex: bigint): Promise<MembershipWitness<typeof NOTE_HASH_TREE_HEIGHT>>;
  },
  noteHashReadRequests: Tuple<ScopedReadRequest, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
  noteHashes: Tuple<ScopedNoteHash, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
  noteHashLeafIndexMap: Map<bigint, bigint>,
) {
  const builder = new NoteHashReadRequestHintsBuilder();

  const numReadRequests = countAccumulatedItems(noteHashReadRequests);

  const noteHashMap: Map<bigint, { noteHash: ScopedNoteHash; index: number }[]> = new Map();
  getNonEmptyItems(noteHashes).forEach((noteHash, index) => {
    const value = noteHash.value.toBigInt();
    const arr = noteHashMap.get(value) ?? [];
    arr.push({ noteHash, index });
    noteHashMap.set(value, arr);
  });

  for (let i = 0; i < numReadRequests; ++i) {
    const readRequest = noteHashReadRequests[i];
    const value = readRequest.value;

    const pendingNoteHash = noteHashMap
      .get(value.toBigInt())
      ?.find(n => isValidNoteHashReadRequest(readRequest, n.noteHash));
    if (pendingNoteHash !== undefined) {
      builder.addPendingReadRequest(i, pendingNoteHash.index);
    } else {
      // TODO(#2847): Read request value for settled note hash shouldn't have been siloed by apps.
      const leafIndex = noteHashLeafIndexMap.get(value.toBigInt());
      if (leafIndex === undefined) {
        throw new Error('Read request is reading an unknown note hash.');
      }
      const membershipWitness = await oracle.getNoteHashMembershipWitness(leafIndex);
      builder.addSettledReadRequest(i, membershipWitness, value);
    }
  }
  return builder.toHints();
}
