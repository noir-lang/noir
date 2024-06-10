import { AztecAddress, KeyValidationRequest, computeOvskApp, derivePublicKeyFromSecretKey } from '@aztec/circuits.js';
import { GrumpkinScalar } from '@aztec/foundation/fields';

import { L1NotePayload } from './l1_note_payload.js';

describe('L1 Note Payload', () => {
  it('convert to and from buffer', () => {
    const payload = L1NotePayload.random();
    const buf = payload.toBuffer();
    expect(L1NotePayload.fromBuffer(buf)).toEqual(payload);
  });

  describe('encrypt and decrypt a full log', () => {
    let ovskM: GrumpkinScalar;
    let ivskM: GrumpkinScalar;

    let payload: L1NotePayload;
    let encrypted: Buffer;

    beforeAll(() => {
      payload = L1NotePayload.random();

      ovskM = GrumpkinScalar.random();
      ivskM = GrumpkinScalar.random();

      const ovKeys = getKeyValidationRequest(ovskM, payload.contractAddress);

      const ephSk = GrumpkinScalar.random();

      const recipientAddress = AztecAddress.random();
      const ivpk = derivePublicKeyFromSecretKey(ivskM);

      encrypted = payload.encrypt(ephSk, recipientAddress, ivpk, ovKeys);
    });

    it('decrypt a log as incoming', () => {
      const recreated = L1NotePayload.decryptAsIncoming(encrypted, ivskM);

      expect(recreated.toBuffer()).toEqual(payload.toBuffer());
    });

    it('decrypt a log as outgoing', () => {
      const recreated = L1NotePayload.decryptAsOutgoing(encrypted, ovskM);

      expect(recreated.toBuffer()).toEqual(payload.toBuffer());
    });
  });

  const getKeyValidationRequest = (ovskM: GrumpkinScalar, app: AztecAddress) => {
    const ovskApp = computeOvskApp(ovskM, app);
    const ovpkM = derivePublicKeyFromSecretKey(ovskM);
    return new KeyValidationRequest(ovpkM, ovskApp);
  };
});
