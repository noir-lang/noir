import { type Wallet } from '@aztec/aztec.js';
import { Aes128 } from '@aztec/circuits.js/barretenberg';
import { TestContract } from '@aztec/noir-contracts.js';

import { randomBytes } from 'crypto';

import { setup } from './fixtures/utils.js';

describe('e2e_encryption', () => {
  const aes128 = new Aes128();

  let wallet: Wallet;
  let teardown: () => Promise<void>;

  let contract: TestContract;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
    contract = await TestContract.deploy(wallet).send().deployed();
  }, 25_000);

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
});
