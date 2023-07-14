import { CircuitsWasm } from '@aztec/circuits.js';

import { createCipheriv, createDecipheriv, randomBytes } from 'crypto';

import { Aes128 } from './index.js';

describe('aes128', () => {
  let barretenberg!: CircuitsWasm;
  let aes128!: Aes128;

  beforeAll(async () => {
    barretenberg = await CircuitsWasm.get();
    aes128 = new Aes128(barretenberg);
  });

  it('should correctly encrypt input', () => {
    const data = randomBytes(32);
    const key = randomBytes(16);
    const iv = randomBytes(16);

    const cipher = createCipheriv('aes-128-cbc', key, iv);
    cipher.setAutoPadding(false);
    const expected = Buffer.concat([cipher.update(data), cipher.final()]);

    const result: Buffer = aes128.encryptBufferCBC(data, iv, key);

    expect(result).toEqual(expected);
  });

  it('should correctly decrypt input', () => {
    const data = randomBytes(32);
    const key = randomBytes(16);
    const iv = randomBytes(16);

    const cipher = createCipheriv('aes-128-cbc', key, iv);
    cipher.setAutoPadding(false);
    const ciphertext = Buffer.concat([cipher.update(data), cipher.final()]);

    const decipher = createDecipheriv('aes-128-cbc', key, iv);
    decipher.setAutoPadding(false);
    const expected = Buffer.concat([decipher.update(ciphertext), decipher.final()]);

    const result: Buffer = aes128.decryptBufferCBC(ciphertext, iv, key);

    expect(result).toEqual(expected);
  });
});
