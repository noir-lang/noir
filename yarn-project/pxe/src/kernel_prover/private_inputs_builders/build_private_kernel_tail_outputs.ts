import {
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  NoteHashContext,
  Nullifier,
  PrivateKernelTailOutputs,
} from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { type Tuple } from '@aztec/foundation/serialize';

export function buildPrivateKernelTailOutputs(
  prevNoteHashes: Tuple<NoteHashContext, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
  prevNullifiers: Tuple<Nullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
) {
  // Propagate note hashes that are not linked to a nullifier.
  // Note that note hashes can't link to the first nullifier (counter == 0).
  const noteHashes = padArrayEnd(
    prevNoteHashes.filter(n => !n.nullifierCounter),
    NoteHashContext.empty(),
    MAX_NEW_NOTE_HASHES_PER_TX,
  );

  const nullifiers = padArrayEnd(
    prevNullifiers.filter(n => n.noteHash.isZero()),
    Nullifier.empty(),
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  return new PrivateKernelTailOutputs(noteHashes, nullifiers);
}
