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
      `"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d460c0e434d846ec1ea286e4090eb56376ff27bddc1aacae1d856549f701fa711a034d916bf54af198535dc02fb2069c6931883ca70958842cdfe0386c36549d413e82a27bfa5b7080712764a455b924510b865903019befeb5df18b7af769fb0873effa97caa035c517a6b417d5f616ec6c84a93d95d17e3543b0f4b6c7a31e6e4f6cfad073c104aecc966ed30b3dfbfdff84ea73dcb1972df3a3cb4ff74aa88adb228027de514dc521cbf938589012df3e58c73a5969a601678dfedd5b6fcc008842b1538f37490b64b101edede3ccd93d635293e3510937548a9dc7dd0d22d41e92857588cedc8a109565280bf3304c3f36466f03681b0748b491b62de017563b233cf431d5368e73189d9f76facf5c6ab7b3929cbdbb187e302bdcd96ee5101cacaf48bc27bc394ffa9e22bea1ffc6923025b0c131a8672b8d25cbfbc07ace8a3bd26c738fbe1caf9117584a2713d4bf6905e6384eb955d03738384faee8ac2e9909c8c012a2c0cd65e89823869957c51b201494f9c1a41a31298748a809e3f97cd974944addc7ed54870ed0febb2d97a92e059d5d922ac04a42866dcaedefadd95eeae6141b7ffa88b437a7c295993ff6d39c596aebadd3213d80e64b0"`,
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
