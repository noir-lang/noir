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
) {
  // Propagate note hashes that are not linked to a nullifier.
  // Note that note hashes can't link to the first nullifier (counter == 0).
  const noteHashes = padArrayEnd(
    prevNoteHashes.filter(n => !n.nullifierCounter),
    ScopedNoteHash.empty(),
    MAX_NEW_NOTE_HASHES_PER_TX,
  );

  const nullifiers = padArrayEnd(
    prevNullifiers.filter(n => n.nullifiedNoteHash.isZero()),
    ScopedNullifier.empty(),
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  const nullifiedNotes = prevNoteHashes.filter(n => !n.isEmpty() && n.nullifierCounter).map(n => n.counter);

  const logs = padArrayEnd(
    prevLogs.filter(l => !l.isEmpty() && !nullifiedNotes.includes(l.noteHashCounter)),
    NoteLogHash.empty(),
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  );

  return new PrivateKernelResetOutputs(noteHashes, nullifiers, logs);
}
