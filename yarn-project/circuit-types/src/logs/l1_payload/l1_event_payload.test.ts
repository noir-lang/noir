import { AztecAddress, KeyValidationRequest, computeOvskApp, derivePublicKeyFromSecretKey } from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';

import { EncryptedL2Log } from '../encrypted_l2_log.js';
import { L1EventPayload } from './l1_event_payload.js';
import { Event } from './payload.js';

describe('L1 Event Payload', () => {
  it('convert to and from buffer', () => {
    const payload = L1EventPayload.random();
    const buf = payload.toBuffer();
    expect(L1EventPayload.fromBuffer(buf)).toEqual(payload);
  });

  describe('encrypt and decrypt a full log', () => {
    let ovskM: GrumpkinScalar;
    let ivskM: GrumpkinScalar;

    let payload: L1EventPayload;
    let encrypted: Buffer;
    let encryptedL2Log: EncryptedL2Log;
    let maskedContractAddress: AztecAddress;
    let contractAddress: AztecAddress;
    let randomness: Fr;

    beforeAll(() => {
      contractAddress = AztecAddress.random();
      randomness = Fr.random();
      maskedContractAddress = pedersenHash([contractAddress, randomness], 0);

      payload = new L1EventPayload(Event.random(), contractAddress, randomness, Fr.random());

      ovskM = GrumpkinScalar.random();
      ivskM = GrumpkinScalar.random();

      const ovKeys = getKeyValidationRequest(ovskM, payload.contractAddress);

      const ephSk = GrumpkinScalar.random();

      const recipientAddress = AztecAddress.random();
      const ivpk = derivePublicKeyFromSecretKey(ivskM);

      encrypted = payload.encrypt(ephSk, recipientAddress, ivpk, ovKeys);
      const tag = Fr.random().toBuffer();
      encryptedL2Log = new EncryptedL2Log(Buffer.concat([tag, tag, encrypted]), maskedContractAddress);
    });

    it('decrypt a log as incoming', () => {
      const recreated = L1EventPayload.decryptAsIncoming(encryptedL2Log, ivskM);

      expect(recreated.toBuffer()).toEqual(payload.toBuffer());
    });

    it('decrypt a log as outgoing', () => {
      const recreated = L1EventPayload.decryptAsOutgoing(encryptedL2Log, ovskM);

      expect(recreated.toBuffer()).toEqual(payload.toBuffer());
    });
  });

  const getKeyValidationRequest = (ovskM: GrumpkinScalar, app: AztecAddress) => {
    const ovskApp = computeOvskApp(ovskM, app);
    const ovpkM = derivePublicKeyFromSecretKey(ovskM);
    return new KeyValidationRequest(ovpkM, ovskApp);
  };
});
