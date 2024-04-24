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

  it('encrypts', async () => {
    const input = randomBytes(64);
    const iv = randomBytes(16);
    const key = randomBytes(16);

    const expectedCiphertext = aes128.encryptBufferCBC(input, iv, key);

    const logs = await contract.methods
      .encrypt(Array.from(input), Array.from(iv), Array.from(key))
      .send()
      .getUnencryptedLogs();
    // Each byte of encrypted data is in its own field and it's all serialized into a long buffer so we simply extract
    // each 32nd byte from the buffer to get the encrypted data
    const recoveredCiphertext = logs.logs[0].log.data.filter((_, i) => (i + 1) % 32 === 0);

    expect(recoveredCiphertext).toEqual(expectedCiphertext);
  });
});
