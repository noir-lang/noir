import { type AztecAddress, type Fr, type NoteSelector } from '@aztec/aztec.js';
import { ExtendedNote, Note, type TxHash } from '@aztec/circuit-types';
import { type DebugLogger } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';
import { parseFields } from '../../utils/commands.js';

export async function addNote(
  address: AztecAddress,
  contractAddress: AztecAddress,
  storageSlot: Fr,
  noteTypeId: NoteSelector,
  txHash: TxHash,
  noteFields: string[],
  rpcUrl: string,
  debugLogger: DebugLogger,
) {
  const note = new Note(parseFields(noteFields));
  const extendedNote = new ExtendedNote(note, address, contractAddress, storageSlot, noteTypeId, txHash);
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  await client.addNote(extendedNote);
}
