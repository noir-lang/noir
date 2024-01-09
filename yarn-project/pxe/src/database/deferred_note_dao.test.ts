import { AztecAddress, Fr, Point } from '@aztec/circuits.js';
import { Note, randomTxHash } from '@aztec/types';

import { DeferredNoteDao } from './deferred_note_dao.js';

export const randomDeferredNoteDao = ({
  publicKey = Point.random(),
  note = Note.random(),
  contractAddress = AztecAddress.random(),
  txHash = randomTxHash(),
  storageSlot = Fr.random(),
  txNullifier = Fr.random(),
  newCommitments = [Fr.random(), Fr.random()],
  dataStartIndexForTx = Math.floor(Math.random() * 100),
}: Partial<DeferredNoteDao> = {}) => {
  return new DeferredNoteDao(
    publicKey,
    note,
    contractAddress,
    storageSlot,
    txHash,
    txNullifier,
    newCommitments,
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
