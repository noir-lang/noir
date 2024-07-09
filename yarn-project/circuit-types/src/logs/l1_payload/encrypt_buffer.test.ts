import { Fq, GrumpkinScalar } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { randomBytes } from '@aztec/foundation/crypto';
import { updateInlineTestData } from '@aztec/foundation/testing';

import { decryptBuffer, encryptBuffer } from './encrypt_buffer.js';
import { deriveAESSecret } from './encryption_utils.js';

describe('encrypt buffer', () => {
  let grumpkin: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('derive shared secret', () => {
    // The following 2 are arbitrary fixed values - fixed in order to test a match with Noir
    const ownerSecretKey = new Fq(0x23b3127c127b1f29a7adff5cccf8fb06649e7ca01d9de27b21624098b897babdn);
    const ephSecretKey = new Fq(0x1fdd0dd8c99b21af8e00d2d130bdc263b36dadcbea84ac5ec9293a0660deca01n);

    const ownerPubKey = grumpkin.mul(Grumpkin.generator, ownerSecretKey);
    const ephPubKey = grumpkin.mul(Grumpkin.generator, ephSecretKey);

    const secretBySender = deriveAESSecret(ephSecretKey, ownerPubKey);
    const secretByReceiver = deriveAESSecret(ownerSecretKey, ephPubKey);
    expect(secretBySender.toString('hex')).toEqual(secretByReceiver.toString('hex'));

    const byteArrayString = `[${secretBySender
      .toString('hex')
      .match(/.{1,2}/g)!
      .map(byte => parseInt(byte, 16))}]`;
    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/keys/point_to_symmetric_key.nr',
      'expected_key',
      byteArrayString,
    );
  });

  it('convert to and from encrypted buffer', () => {
    const data = randomBytes(253);
    const ownerSecretKey = GrumpkinScalar.random();
    const ownerPubKey = grumpkin.mul(Grumpkin.generator, ownerSecretKey);
    const ephSecretKey = GrumpkinScalar.random();
    const encrypted = encryptBuffer(data, ephSecretKey, ownerPubKey);
    const decrypted = decryptBuffer(encrypted, ownerSecretKey);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted).toEqual(data);
  });

  it('decrypting gibberish returns undefined', () => {
    const data = randomBytes(253);
    const ownerSecretKey = GrumpkinScalar.random();
    const ephSecretKey = GrumpkinScalar.random();
    const ownerPubKey = grumpkin.mul(Grumpkin.generator, ownerSecretKey);
    const encrypted = encryptBuffer(data, ephSecretKey, ownerPubKey);

    // Introduce gibberish.
    const gibberish = Buffer.concat([randomBytes(8), encrypted.subarray(8)]);

    const decrypted = decryptBuffer(gibberish, ownerSecretKey);
    expect(decrypted).toBeUndefined();
  });
});
