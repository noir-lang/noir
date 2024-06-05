import { Note, randomTxHash } from '@aztec/circuit-types';
import { AztecAddress, Fr, Point } from '@aztec/circuits.js';
import { randomInt } from '@aztec/foundation/crypto';

import { DeferredNoteDao } from './deferred_note_dao.js';

export const randomDeferredNoteDao = ({
  ivpkM = Point.random(),
  note = Note.random(),
  contractAddress = AztecAddress.random(),
  txHash = randomTxHash(),
  storageSlot = Fr.random(),
  noteTypeId = Fr.random(),
  newNoteHashes = [Fr.random(), Fr.random()],
  dataStartIndexForTx = randomInt(100),
}: Partial<DeferredNoteDao> = {}) => {
  return new DeferredNoteDao(
    ivpkM,
    note,
    contractAddress,
    storageSlot,
    noteTypeId,
    txHash,
    newNoteHashes,
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
