import { Note, randomTxHash } from '@aztec/circuit-types';
import { AztecAddress, Fr, Point } from '@aztec/circuits.js';

import { IncomingNoteDao } from './incoming_note_dao.js';

export const randomIncomingNoteDao = ({
  note = Note.random(),
  contractAddress = AztecAddress.random(),
  txHash = randomTxHash(),
  storageSlot = Fr.random(),
  noteTypeId = Fr.random(),
  nonce = Fr.random(),
  innerNoteHash = Fr.random(),
  siloedNullifier = Fr.random(),
  index = Fr.random().toBigInt(),
  ivpkM = Point.random(),
}: Partial<IncomingNoteDao> = {}) => {
  return new IncomingNoteDao(
    note,
    contractAddress,
    storageSlot,
    noteTypeId,
    txHash,
    nonce,
    innerNoteHash,
    siloedNullifier,
    index,
    ivpkM,
  );
};

describe('Incoming Note DAO', () => {
  it('convert to and from buffer', () => {
    const note = randomIncomingNoteDao();
    const buf = note.toBuffer();
    expect(IncomingNoteDao.fromBuffer(buf)).toEqual(note);
  });
});
