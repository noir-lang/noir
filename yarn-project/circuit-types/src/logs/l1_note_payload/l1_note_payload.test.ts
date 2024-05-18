import { AztecAddress } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { GrumpkinScalar } from '@aztec/foundation/fields';

import { L1NotePayload } from './l1_note_payload.js';

describe('L1 Note Payload', () => {
  let grumpkin: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('convert to and from buffer', () => {
    const payload = L1NotePayload.random();
    const buf = payload.toBuffer();
    expect(L1NotePayload.fromBuffer(buf)).toEqual(payload);
  });

  describe('encrypt and decrypt a full log', () => {
    let ovsk: GrumpkinScalar;
    let ivsk: GrumpkinScalar;

    let payload: L1NotePayload;
    let encrypted: Buffer;

    beforeAll(() => {
      ovsk = GrumpkinScalar.random();
      ivsk = GrumpkinScalar.random();

      const ephSk = GrumpkinScalar.random();

      const recipientAddress = AztecAddress.random();
      const ivpk = grumpkin.mul(Grumpkin.generator, ivsk);

      payload = L1NotePayload.random();
      encrypted = payload.encrypt(ephSk, recipientAddress, ivpk, ovsk);
    });

    it('decrypt a log as incoming', () => {
      const recreated = L1NotePayload.decryptAsIncoming(encrypted, ivsk);

      expect(recreated.toBuffer()).toEqual(payload.toBuffer());
    });

    it('decrypt a log as outgoing', () => {
      const recreated = L1NotePayload.decryptAsOutgoing(encrypted, ovsk);

      expect(recreated.toBuffer()).toEqual(payload.toBuffer());
    });
  });
});
