import {
  type ScopedNoteHash,
  type ScopedNullifier,
  type ScopedReadRequest,
  countAccumulatedItems,
  isValidNoteHashReadRequest,
  isValidNullifierReadRequest,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';

import { ScopedValueCache } from './scoped_value_cache.js';

export function buildTransientDataHints<NOTE_HASHES_LEN extends number, NULLIFIERS_LEN extends number>(
  noteHashes: Tuple<ScopedNoteHash, NOTE_HASHES_LEN>,
  nullifiers: Tuple<ScopedNullifier, NULLIFIERS_LEN>,
  futureNoteHashReads: ScopedReadRequest[],
  futureNullifierReads: ScopedReadRequest[],
  noteHashNullifierCounterMap: Map<number, number>,
  noteHashesLength: NOTE_HASHES_LEN = noteHashes.length as NOTE_HASHES_LEN,
  nullifiersLength: NULLIFIERS_LEN = nullifiers.length as NULLIFIERS_LEN,
): [Tuple<number, NOTE_HASHES_LEN>, Tuple<number, NULLIFIERS_LEN>] {
  const futureNoteHashReadsMap = new ScopedValueCache(futureNoteHashReads);
  const futureNullifierReadsMap = new ScopedValueCache(futureNullifierReads);

  const nullifierIndexMap: Map<number, number> = new Map();
  nullifiers.forEach((n, i) => nullifierIndexMap.set(n.counter, i));

  const nullifierIndexesForNoteHashes: Tuple<number, NOTE_HASHES_LEN> = makeTuple(
    noteHashesLength,
    () => nullifiersLength,
  );

  const noteHashIndexesForNullifiers: Tuple<number, NULLIFIERS_LEN> = makeTuple(
    nullifiersLength,
    () => noteHashesLength,
  );

  const numNoteHashes = countAccumulatedItems(noteHashes);
  for (let i = 0; i < numNoteHashes; i++) {
    const noteHash = noteHashes[i];
    const noteHashNullifierCounter = noteHashNullifierCounterMap.get(noteHash.counter);
    // The note hash might not be linked to a nullifier or it might be read in the future
    if (
      !noteHashNullifierCounter ||
      futureNoteHashReadsMap.get(noteHash).find(read => isValidNoteHashReadRequest(read, noteHash))
    ) {
      continue;
    }

    const nullifierIndex = nullifierIndexMap.get(noteHashNullifierCounter);
    // We might not have the corresponding nullifier yet
    if (nullifierIndex === undefined) {
      continue;
    }

    const nullifier = nullifiers[nullifierIndex];

    if (!nullifier.nullifiedNoteHash.equals(noteHash.value)) {
      throw new Error('Hinted note hash does not match.');
    }
    if (!nullifier.contractAddress.equals(noteHash.contractAddress)) {
      throw new Error('Contract address of hinted note hash does not match.');
    }

    // The nullifier might be read in the future
    if (futureNullifierReadsMap.get(nullifier).find(read => isValidNullifierReadRequest(read, nullifier))) {
      continue;
    }

    nullifierIndexesForNoteHashes[i] = nullifierIndex;
    noteHashIndexesForNullifiers[nullifierIndex] = i;
  }

  return [nullifierIndexesForNoteHashes, noteHashIndexesForNullifiers];
}
