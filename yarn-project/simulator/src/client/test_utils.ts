import { Fq, Fr, GeneratorIndex, Point } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { pedersenCommit } from '@aztec/foundation/crypto';

// Copied over from `noir-projects/aztec-nr/aztec/src/generators.nr`
const G_SLOT = new Point(
  new Fr(0x041223147b680850dc82e8a55a952d4df20256fe0593d949a9541ca00f0abf15n),
  new Fr(0x0a8c72e60d0e60f5d804549d48f3044d06140b98ed717a9b532af630c1530791n),
  false,
);

/**
 * Computes a note hiding point as is done by the default implementation injected by macros.
 * @param storageSlot - The slot to which the note was inserted.
 * @param noteContent - The note content (e.g. note.items).
 * @returns A note hash.
 */
export function computeNoteHash(storageSlot: Fr, noteContent: Fr[]): Fr {
  // TODO(#7771): update this to do only 1 MSM call
  const c = pedersenCommit(
    noteContent.map(f => f.toBuffer()),
    GeneratorIndex.NOTE_HIDING_POINT,
  );
  const noteHidingPointBeforeSlotting = new Point(new Fr(c[0]), new Fr(c[1]), false);

  const grumpkin = new Grumpkin();
  const slotPoint = grumpkin.mul(G_SLOT, new Fq(storageSlot.toBigInt()));
  const noteHidingPoint = grumpkin.add(noteHidingPointBeforeSlotting, slotPoint);
  return noteHidingPoint.x;
}
