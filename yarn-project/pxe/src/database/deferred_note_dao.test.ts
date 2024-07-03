import { Note, randomTxHash } from '@aztec/circuit-types';
import { AztecAddress, Fr, Point } from '@aztec/circuits.js';
import { NoteSelector } from '@aztec/foundation/abi';
import { randomInt } from '@aztec/foundation/crypto';

import { DeferredNoteDao } from './deferred_note_dao.js';

export const randomDeferredNoteDao = ({
  publicKey = Point.random(),
  note = Note.random(),
  contractAddress = AztecAddress.random(),
  txHash = randomTxHash(),
  storageSlot = Fr.random(),
  noteTypeId = NoteSelector.random(),
  noteHashes = [Fr.random(), Fr.random()],
  dataStartIndexForTx = randomInt(100),
}: Partial<DeferredNoteDao> = {}) => {
  return new DeferredNoteDao(
    publicKey,
    note,
    contractAddress,
    storageSlot,
    noteTypeId,
    txHash,
    noteHashes,
    dataStartIndexForTx,
  );
};

describe('Deferred Note DAO', () => {
  it('convert to and from buffer', () => {
    const deferredNote = randomDeferredNoteDao();
    const buf = deferredNote.toBuffer();
    expect(DeferredNoteDao.fromBuffer(buf)).toEqual(deferredNote);
  });
});
