import {
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  NoteLogHash,
  PrivateKernelResetOutputs,
  ScopedNoteHash,
  ScopedNullifier,
} from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { type Tuple } from '@aztec/foundation/serialize';

export function buildPrivateKernelResetOutputs(
  prevNoteHashes: Tuple<ScopedNoteHash, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
  prevNullifiers: Tuple<ScopedNullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  prevLogs: Tuple<NoteLogHash, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
  transientNullifierIndexesForNoteHashes: Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
  transientNoteHashIndexesForNullifiers: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
) {
  // Propagate note hashes that are not going to be squashed in the transient arrays.
  // A value isn't going to be squashed if the symmetrical index in the corresponding array is the length of the array.
  const noteHashes = padArrayEnd(
    prevNoteHashes.filter((_, index) => transientNullifierIndexesForNoteHashes[index] === MAX_NEW_NULLIFIERS_PER_TX),
    ScopedNoteHash.empty(),
    MAX_NEW_NOTE_HASHES_PER_TX,
  );

  const nullifiers = padArrayEnd(
    prevNullifiers.filter((_, index) => transientNoteHashIndexesForNullifiers[index] === MAX_NEW_NOTE_HASHES_PER_TX),
    ScopedNullifier.empty(),
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  const nullifiedNotes = prevNoteHashes
    .filter((_, index) => transientNullifierIndexesForNoteHashes[index] < MAX_NEW_NULLIFIERS_PER_TX)
    .map(n => n.counter);

  const logs = padArrayEnd(
    prevLogs.filter(l => !l.isEmpty() && !nullifiedNotes.includes(l.noteHashCounter)),
    NoteLogHash.empty(),
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  );

  return new PrivateKernelResetOutputs(noteHashes, nullifiers, logs);
}
