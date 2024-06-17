import { Note, randomTxHash } from '@aztec/circuit-types';
import { AztecAddress, Fr, Point } from '@aztec/circuits.js';

import { OutgoingNoteDao } from './outgoing_note_dao.js';

export const randomOutgoingNoteDao = ({
  note = Note.random(),
  contractAddress = AztecAddress.random(),
  txHash = randomTxHash(),
  storageSlot = Fr.random(),
  noteTypeId = Fr.random(),
  nonce = Fr.random(),
  innerNoteHash = Fr.random(),
  index = Fr.random().toBigInt(),
  ovpkM = Point.random(),
}: Partial<OutgoingNoteDao> = {}) => {
  return new OutgoingNoteDao(
    note,
    contractAddress,
    storageSlot,
    noteTypeId,
    txHash,
    nonce,
    innerNoteHash,
    index,
    ovpkM,
  );
};

describe('Outgoing Note DAO', () => {
  it('convert to and from buffer', () => {
    const note = randomOutgoingNoteDao();
    const buf = note.toBuffer();
    expect(OutgoingNoteDao.fromBuffer(buf)).toEqual(note);
  });
});
