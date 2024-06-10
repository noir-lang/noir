import { AztecAddress, KeyValidationRequest, computeOvskApp, derivePublicKeyFromSecretKey } from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';

import { EncryptedL2Log } from '../encrypted_l2_log.js';
import { L1EventPayload } from './l1_event_payload.js';
import { L1NotePayload } from './l1_note_payload.js';
import { Event } from './payload.js';
import { TaggedLog } from './tagged_log.js';

describe('L1 Note Payload', () => {
  it('convert to and from buffer', () => {
    const payload = L1NotePayload.random();
    const taggedLog = new TaggedLog(payload);
    const buf = taggedLog.toBuffer();
    expect(TaggedLog.fromBuffer(buf).payload).toEqual(taggedLog.payload);
  });

  describe('encrypt and decrypt a full log', () => {
    let ovskM: GrumpkinScalar;
    let ivskM: GrumpkinScalar;

    let taggedLog: TaggedLog<L1NotePayload>;
    let encrypted: Buffer;

    beforeAll(() => {
      const payload = L1NotePayload.random();

      ovskM = GrumpkinScalar.random();
      ivskM = GrumpkinScalar.random();

      const ovKeys = getKeyValidationRequest(ovskM, payload.contractAddress);

      const ephSk = GrumpkinScalar.random();

      const recipientAddress = AztecAddress.random();
      const ivpk = derivePublicKeyFromSecretKey(ivskM);

      taggedLog = new TaggedLog(payload);

      encrypted = taggedLog.encrypt(ephSk, recipientAddress, ivpk, ovKeys);
    });

    it('decrypt a log as incoming', () => {
      const recreated = TaggedLog.decryptAsIncoming(encrypted, ivskM);

      expect(recreated?.toBuffer()).toEqual(taggedLog.toBuffer());
    });

    it('decrypt a log as outgoing', () => {
      const recreated = TaggedLog.decryptAsOutgoing(encrypted, ovskM);

      expect(recreated?.toBuffer()).toEqual(taggedLog.toBuffer());
    });
  });

  const getKeyValidationRequest = (ovskM: GrumpkinScalar, app: AztecAddress) => {
    const ovskApp = computeOvskApp(ovskM, app);
    const ovpkM = derivePublicKeyFromSecretKey(ovskM);

    return new KeyValidationRequest(ovpkM, ovskApp);
  };
});

describe('L1 Event Payload', () => {
  it('convert to and from buffer', () => {
    const payload = L1EventPayload.random();
    const taggedLog = new TaggedLog(payload);
    const buf = taggedLog.toBuffer();
    expect(TaggedLog.fromBuffer(buf, L1EventPayload).payload).toEqual(taggedLog.payload);
  });

  describe('encrypt and decrypt a full log', () => {
    let ovskM: GrumpkinScalar;
    let ivskM: GrumpkinScalar;

    let taggedLog: TaggedLog<L1EventPayload>;
    let encrypted: Buffer;
    let maskedContractAddress: AztecAddress;
    let contractAddress: AztecAddress;
    let randomness: Fr;
    let encryptedL2Log: EncryptedL2Log;

    beforeAll(() => {
      contractAddress = AztecAddress.random();
      randomness = Fr.random();
      maskedContractAddress = pedersenHash([contractAddress, randomness], 0);

      const payload = new L1EventPayload(Event.random(), contractAddress, randomness, Fr.random());

      ovskM = GrumpkinScalar.random();
      ivskM = GrumpkinScalar.random();

      const ovKeys = getKeyValidationRequest(ovskM, payload.contractAddress);

      const ephSk = GrumpkinScalar.random();

      const recipientAddress = AztecAddress.random();
      const ivpk = derivePublicKeyFromSecretKey(ivskM);

      taggedLog = new TaggedLog(payload);

      encrypted = taggedLog.encrypt(ephSk, recipientAddress, ivpk, ovKeys);
      encryptedL2Log = new EncryptedL2Log(encrypted, maskedContractAddress);
    });

    it('decrypt a log as incoming', () => {
      const recreated = TaggedLog.decryptAsIncoming(encryptedL2Log, ivskM, L1EventPayload);

      expect(recreated?.toBuffer()).toEqual(taggedLog.toBuffer());
    });

    it('decrypt a log as outgoing', () => {
      const recreated = TaggedLog.decryptAsOutgoing(encryptedL2Log, ovskM, L1EventPayload);

      expect(recreated?.toBuffer()).toEqual(taggedLog.toBuffer());
    });
  });

  const getKeyValidationRequest = (ovskM: GrumpkinScalar, app: AztecAddress) => {
    const ovskApp = computeOvskApp(ovskM, app);
    const ovpkM = derivePublicKeyFromSecretKey(ovskM);

    return new KeyValidationRequest(ovpkM, ovskApp);
  };
});
