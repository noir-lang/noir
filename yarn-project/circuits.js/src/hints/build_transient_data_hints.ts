import { type NoteHashContext, type Nullifier, countAccumulatedItems } from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';

export function buildTransientDataHints<NOTE_HASHES_LEN extends number, NULLIFIERS_LEN extends number>(
  noteHashes: Tuple<NoteHashContext, NOTE_HASHES_LEN>,
  nullifiers: Tuple<Nullifier, NULLIFIERS_LEN>,
  noteHashesLength: NOTE_HASHES_LEN = noteHashes.length as NOTE_HASHES_LEN,
  nullifiersLength: NULLIFIERS_LEN = nullifiers.length as NULLIFIERS_LEN,
): [Tuple<number, NOTE_HASHES_LEN>, Tuple<number, NULLIFIERS_LEN>] {
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
    if (noteHash.nullifierCounter > 0) {
      const nullifierIndex = nullifierIndexMap.get(noteHash.nullifierCounter);
      if (nullifierIndex === undefined) {
        throw new Error('Unknown nullifier counter.');
      }

      const nullifier = nullifiers[nullifierIndex];
      if (!nullifier.noteHash.equals(noteHash.value)) {
        throw new Error('Hinted note hash does not match.');
      }

      nullifierIndexesForNoteHashes[i] = nullifierIndex;
      noteHashIndexesForNullifiers[nullifierIndex] = i;
    }
  }

  return [nullifierIndexesForNoteHashes, noteHashIndexesForNullifiers];
}
