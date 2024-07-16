import {
  AztecAddress,
  EncryptedLogHeader,
  EncryptedLogOutgoingBody,
  EncryptedNoteLogIncomingBody,
  Fr,
  GrumpkinScalar,
  Note,
  type Wallet,
} from '@aztec/aztec.js';
import { Aes128, Grumpkin } from '@aztec/circuits.js/barretenberg';
import { TestContract } from '@aztec/noir-contracts.js';

import { randomBytes } from 'crypto';

import { setup } from './fixtures/utils.js';

describe('e2e_encryption', () => {
  const aes128 = new Aes128();
  let grumpkin: Grumpkin;

  let wallet: Wallet;
  let teardown: () => Promise<void>;

  let contract: TestContract;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
    contract = await TestContract.deploy(wallet).send().deployed();
    grumpkin = new Grumpkin();
  }, 120_000);

  afterAll(() => teardown());

  it('encrypts 🔒📄🔑💻', async () => {
    const input = randomBytes(64);
    const iv = randomBytes(16);
    const key = randomBytes(16);

    const expectedCiphertext = aes128.encryptBufferCBC(input, iv, key);

    const ciphertextAsBigInts = await contract.methods
      .encrypt(Array.from(input), Array.from(iv), Array.from(key))
      .simulate();
    const ciphertext = Buffer.from(ciphertextAsBigInts.map((x: bigint) => Number(x)));

    expect(ciphertext).toEqual(expectedCiphertext);
  });

  it('encrypts with padding 🔒📄🔑💻 ➕ 📦', async () => {
    const input = randomBytes(65);
    const iv = randomBytes(16);
    const key = randomBytes(16);

    const expectedCiphertext = aes128.encryptBufferCBC(input, iv, key);
    // AES 128 CBC with PKCS7 is padding to multiples of 16 bytes so from 65 bytes long input we get 80 bytes long output
    expect(expectedCiphertext.length).toBe(80);

    const ciphertextAsBigInts = await contract.methods
      .encrypt_with_padding(Array.from(input), Array.from(iv), Array.from(key))
      .simulate();
    const ciphertext = Buffer.from(ciphertextAsBigInts.map((x: bigint) => Number(x)));

    expect(ciphertext).toEqual(expectedCiphertext);
  });

  it('encrypts log header', async () => {
    const ephSecretKey = GrumpkinScalar.random();
    const viewingSecretKey = GrumpkinScalar.random();

    const ephPubKey = grumpkin.mul(Grumpkin.generator, ephSecretKey);
    const viewingPubKey = grumpkin.mul(Grumpkin.generator, viewingSecretKey);
    const header = new EncryptedLogHeader(contract.address);

    const encrypted = await contract.methods
      .compute_note_header_ciphertext(ephSecretKey, viewingPubKey.toNoirStruct())
      .simulate();
    expect(Buffer.from(encrypted.map((x: bigint) => Number(x)))).toEqual(
      header.computeCiphertext(ephSecretKey, viewingPubKey),
    );

    const recreated = EncryptedLogHeader.fromCiphertext(encrypted, viewingSecretKey, ephPubKey);

    expect(recreated.address).toEqual(contract.address);
  });

  it('encrypts log incoming body', async () => {
    const ephSecretKey = GrumpkinScalar.random();
    const viewingSecretKey = GrumpkinScalar.random();

    const ephPubKey = grumpkin.mul(Grumpkin.generator, ephSecretKey);
    const viewingPubKey = grumpkin.mul(Grumpkin.generator, viewingSecretKey);

    const storageSlot = new Fr(1);
    const noteTypeId = TestContract.artifact.notes['TestNote'].id;
    const value = Fr.random();
    const note = new Note([value]);

    const body = new EncryptedNoteLogIncomingBody(storageSlot, noteTypeId, note);

    const encrypted = await contract.methods
      .compute_incoming_log_body_ciphertext(ephSecretKey, viewingPubKey.toNoirStruct(), storageSlot, value)
      .simulate();

    expect(Buffer.from(encrypted.map((x: bigint) => Number(x)))).toEqual(
      body.computeCiphertext(ephSecretKey, viewingPubKey),
    );

    const recreated = EncryptedNoteLogIncomingBody.fromCiphertext(encrypted, viewingSecretKey, ephPubKey);

    expect(recreated.toBuffer()).toEqual(body.toBuffer());
  });

  it('encrypts log outgoing body', async () => {
    const ephSk = GrumpkinScalar.random();
    const recipientIvsk = GrumpkinScalar.random();
    const senderOvskApp = GrumpkinScalar.random();

    const ephPk = grumpkin.mul(Grumpkin.generator, ephSk);
    const recipientIvpk = grumpkin.mul(Grumpkin.generator, recipientIvsk);

    const recipientAddress = AztecAddress.fromBigInt(BigInt('0xdeadbeef'));

    const body = new EncryptedLogOutgoingBody(ephSk, recipientAddress, recipientIvpk);

    const encrypted = await contract.methods
      .compute_outgoing_log_body_ciphertext(ephSk, recipientAddress, recipientIvpk.toNoirStruct(), senderOvskApp)
      .simulate();

    expect(Buffer.from(encrypted.map((x: bigint) => Number(x)))).toEqual(body.computeCiphertext(senderOvskApp, ephPk));

    const recreated = EncryptedLogOutgoingBody.fromCiphertext(encrypted, senderOvskApp, ephPk);

    expect(recreated.toBuffer()).toEqual(body.toBuffer());
  });
});
