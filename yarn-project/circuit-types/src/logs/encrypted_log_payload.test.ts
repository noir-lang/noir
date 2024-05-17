import { AztecAddress, GrumpkinScalar } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';

import { EncryptedLogPayload } from './encrypted_log_payload.js';
import { L1NotePayload } from './l1_note_payload/l1_note_payload.js';

describe('encrypt and decrypt a full log', () => {
  let grumpkin: Grumpkin;

  let ovsk: GrumpkinScalar;
  let ivsk: GrumpkinScalar;

  let payload: EncryptedLogPayload;
  let encrypted: Buffer;

  beforeAll(() => {
    grumpkin = new Grumpkin();

    ovsk = GrumpkinScalar.random();
    ivsk = GrumpkinScalar.random();

    const ephSk = GrumpkinScalar.random();

    const recipientAddress = AztecAddress.random();
    const ivpk = grumpkin.mul(Grumpkin.generator, ivsk);

    payload = EncryptedLogPayload.fromL1NotePayload(L1NotePayload.random());
    encrypted = payload.encrypt(ephSk, recipientAddress, ivpk, ovsk);
  });

  it('decrypt a log as incoming', () => {
    const recreated = EncryptedLogPayload.decryptAsIncoming(encrypted, ivsk);

    expect(recreated.toBuffer()).toEqual(payload.toBuffer());
  });

  it('decrypt a log as outgoing', () => {
    const recreated = EncryptedLogPayload.decryptAsOutgoing(encrypted, ovsk);

    expect(recreated.toBuffer()).toEqual(payload.toBuffer());
  });
});
