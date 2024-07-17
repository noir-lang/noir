import { AztecAddress, KeyValidationRequest, computeOvskApp, derivePublicKeyFromSecretKey } from '@aztec/circuits.js';
import { EventSelector, NoteSelector } from '@aztec/foundation/abi';
import { pedersenHash } from '@aztec/foundation/crypto';
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
    const contract = AztecAddress.fromString('0x10f48cd9eff7ae5b209c557c70de2e657ee79166868676b787e9417e19260e04');
    const storageSlot = new Fr(0x0fe46be583b71f4ab5b70c2657ff1d05cccf1d292a9369628d1a194f944e6599n);
    const noteValue = new Fr(0x301640ceea758391b2e161c92c0513f129020f4125256afdae2646ce31099f5cn);
    const noteTypeId = new NoteSelector(0);

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
      `"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008d460c0e434d846ec1ea286e4090eb56376ff27bddc1aacae1d856549f701fa79f357275ed3983136f963253ad9beae147bb8d4ff52b6f53db957c440cf4fdd8003e6ce87650578cd7b96f3080ec6e5c2ecd07e28342cd006753d95a3c8a06acf6815cac45494d419312e71423d9b4fd48f220392d0b02eb1860f4e0213d97e188adb228027de514dc521cbf938589012df3e58c73a5969a601678dfedd5b6fcc008842b1538f37490b64b101edede3ccd93d635293e3510937548a9dc7dd0d22d41e92857588cedc8a109565280bf3304c3f36466f03681b0748b491b62de01f3c748eed5425b9fb78f24675e053e320dd9a14f1ee729e46d8bf377a63625fac106431d94b9993a40d2a4dba550234b6db10ea8886915eb3e9f473df5c1eaa964a508de9def29dddf43503dfc361b64016802793e2917840f7c7815c67197ac2aa140f0a6cd50a93abf6f82373a8d1a617672d845cfd4e3fac7154890552b4cd51c848610dd697052ee723d2490b3b244c6a2d4556474ba83e821e565fb05fb"`,
    );

    const byteArrayString = `[${encrypted.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))}]`;

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/encrypted_logs/payload.nr',
      'expected_encrypted_note_log',
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
      maskedContractAddress = pedersenHash([contractAddress, randomness], 0);

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
