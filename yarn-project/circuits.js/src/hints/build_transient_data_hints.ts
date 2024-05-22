import {
  type NoteLogHash,
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

export function buildTransientDataHints<
  NOTE_HASHES_LEN extends number,
  NULLIFIERS_LEN extends number,
  LOGS_LEN extends number,
>(
  noteHashes: Tuple<ScopedNoteHash, NOTE_HASHES_LEN>,
  nullifiers: Tuple<ScopedNullifier, NULLIFIERS_LEN>,
  noteLogs: Tuple<NoteLogHash, LOGS_LEN>,
  futureNoteHashReads: ScopedReadRequest[],
  futureNullifierReads: ScopedReadRequest[],
  noteHashesLength: NOTE_HASHES_LEN = noteHashes.length as NOTE_HASHES_LEN,
  nullifiersLength: NULLIFIERS_LEN = nullifiers.length as NULLIFIERS_LEN,
  logsLength: LOGS_LEN = noteLogs.length as LOGS_LEN,
): [Tuple<number, NOTE_HASHES_LEN>, Tuple<number, NULLIFIERS_LEN>, Tuple<number, LOGS_LEN>] {
  const futureNoteHashReadsMap = new ScopedValueCache(futureNoteHashReads);
  const futureNullifierReadsMap = new ScopedValueCache(futureNullifierReads);

  const nullifierIndexMap: Map<number, number> = new Map();
  nullifiers.forEach((n, i) => nullifierIndexMap.set(n.counter, i));

  const logNoteHashMap: Map<number, number[]> = new Map();
  noteLogs.forEach((n, i) => {
    logNoteHashMap.set(n.noteHashCounter, (logNoteHashMap.get(n.noteHashCounter) || []).concat([i]));
  });

  const nullifierIndexesForNoteHashes: Tuple<number, NOTE_HASHES_LEN> = makeTuple(
    noteHashesLength,
    () => nullifiersLength,
  );

  const noteHashIndexesForNullifiers: Tuple<number, NULLIFIERS_LEN> = makeTuple(
    nullifiersLength,
    () => noteHashesLength,
  );

  const noteHashIndexesForLogs: Tuple<number, LOGS_LEN> = makeTuple(logsLength, () => noteHashesLength);

  const numNoteHashes = countAccumulatedItems(noteHashes);
  for (let i = 0; i < numNoteHashes; i++) {
    const noteHash = noteHashes[i];
    // The note hash might not be linked to a nullifier or it might be read in the future
    if (
      noteHash.nullifierCounter == 0 ||
      futureNoteHashReadsMap.get(noteHash).find(read => isValidNoteHashReadRequest(read, noteHash))
    ) {
      continue;
    }
    const nullifierIndex = nullifierIndexMap.get(noteHash.nullifierCounter);

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

    const logIndices = logNoteHashMap.get(noteHash.counter);
    if (logIndices) {
      logIndices.forEach(logIndex => {
        noteHashIndexesForLogs[logIndex] = i;
      });
    }

    nullifierIndexesForNoteHashes[i] = nullifierIndex;
    noteHashIndexesForNullifiers[nullifierIndex] = i;
  }

  return [nullifierIndexesForNoteHashes, noteHashIndexesForNullifiers, noteHashIndexesForLogs];
}
