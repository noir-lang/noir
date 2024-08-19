import { AztecAddress, KeyValidationRequest, computeOvskApp, derivePublicKeyFromSecretKey } from '@aztec/circuits.js';
import { EventSelector, NoteSelector } from '@aztec/foundation/abi';
import { poseidon2HashWithSeparator } from '@aztec/foundation/crypto';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';
import { updateInlineTestData } from '@aztec/foundation/testing';

import { EncryptedL2Log } from '../encrypted_l2_log.js';
import { L1EventPayload } from './l1_event_payload.js';
import { L1NotePayload } from './l1_note_payload.js';
import { Event, Note } from './payload.js';
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

  it('encrypted tagged log matches Noir', () => {
    // All the values in this test were arbitrarily set and copied over to `payload.nr`
    const contract = AztecAddress.fromString('0x10f48cd9eff7ae5b209c557c70de2e657ee79166868676b787e9417e19260e04');
    const storageSlot = new Fr(0x0fe46be583b71f4ab5b70c2657ff1d05cccf1d292a9369628d1a194f944e6599n);
    const noteValue = new Fr(0x301640ceea758391b2e161c92c0513f129020f4125256afdae2646ce31099f5cn);
    const noteTypeId = new NoteSelector(4135); // note type id of mock_note.nr

    const payload = new L1NotePayload(new Note([noteValue]), contract, storageSlot, noteTypeId);

    const ovskM = new GrumpkinScalar(0x06b76394ac57b8a18ceb08b14ed15b5b778d5c506b4cfb7edc203324eab27c05n);
    const ivskM = new GrumpkinScalar(0x03fd94b1101e834e829cda4f227043f60490b5c7b3073875f5bfbe5028ed05ccn);

    const ovKeys = getKeyValidationRequest(ovskM, payload.contractAddress);

    const ephSk = new GrumpkinScalar(0x1358d15019d4639393d62b97e1588c095957ce74a1c32d6ec7d62fe6705d9538n);

    const recipientAddress = AztecAddress.fromString(
      '0x10ee41ee4b62703b16f61e03cb0d88c4b306a9eb4a6ceeb2aff13428541689a2',
    );

    const ivpk = derivePublicKeyFromSecretKey(ivskM);

    const taggedLog = new TaggedLog(payload, new Fr(0), new Fr(0));

    const encrypted = taggedLog.encrypt(ephSk, recipientAddress, ivpk, ovKeys).toString('hex');

    expect(encrypted).toMatchInlineSnapshot(
      `"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008d460c0e434d846ec1ea286e4090eb56376ff27bddc1aacae1d856549f701fa77e4f33ba2f47fdac6370f13bc5f16bbae857bbe6ab3ee4ea2a339192eef22a47ce0df4426fc314cb6294ccf291b79c1d8d362cdcc223e51020ccd3318e7052ca74f1fe922ad914bd46e4b6abcd681b63ab1c5bf4151e82f00548ae7c61c59df8c117c14c2e8d9046d32d43a7da818c68be296ef9d1446a87a450eb3f6550200d2663915b0bad97e7f7419975e5a740efb67eeb5304a90808a004ebfc156054a1459191d7fea175f6c64159b3c25a13790cca7250c30e3c80698e64565a6c9ddb16ac1479c3199fec02464b2a252202119514b02012cc387579220f03587b40444ae93f3b83dec2c0a76ed90a804981accd67d43c978d0a97de97b42b5b94c96ea50aee2086eb63d8c8b61f169c12d1deacefc1d456633e46b62daff15bcab3e1ec5f474297e1cb35d8556682060819b4563a8cc66966b12a5e73f7919318e727491b0adb8273bc4a7205b1c753b76a57cceee7482df027ae196235bb9c9ff426"`,
    );

    const byteArrayString = `[${encrypted.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))}]`;

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/encrypted_logs/payload.nr',
      'encrypted_note_log_from_typescript',
      byteArrayString,
    );
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
      maskedContractAddress = poseidon2HashWithSeparator([contractAddress, randomness], 0);

      const payload = new L1EventPayload(Event.random(), contractAddress, randomness, EventSelector.random());

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
