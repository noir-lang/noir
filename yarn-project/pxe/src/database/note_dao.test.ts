import { Note, randomTxHash } from '@aztec/circuit-types';
import { AztecAddress, Fr, Point } from '@aztec/circuits.js';

import { NoteDao } from './note_dao.js';

export const randomNoteDao = ({
  note = Note.random(),
  contractAddress = AztecAddress.random(),
  txHash = randomTxHash(),
  storageSlot = Fr.random(),
  nonce = Fr.random(),
  innerNoteHash = Fr.random(),
  siloedNullifier = Fr.random(),
  index = Fr.random().toBigInt(),
  publicKey = Point.random(),
}: Partial<NoteDao> = {}) => {
  return new NoteDao(
    note,
    contractAddress,
    storageSlot,
    txHash,
    nonce,
    innerNoteHash,
    siloedNullifier,
    index,
    publicKey,
  );
};

describe('Note DAO', () => {
  it('convert to and from buffer', () => {
    const note = randomNoteDao();
    const buf = note.toBuffer();
    expect(NoteDao.fromBuffer(buf)).toEqual(note);
  });
});
