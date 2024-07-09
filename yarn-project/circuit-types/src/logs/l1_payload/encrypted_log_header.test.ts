import { AztecAddress, GrumpkinScalar } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { updateInlineTestData } from '@aztec/foundation/testing';

import { EncryptedLogHeader } from './encrypted_log_header.js';

describe('encrypt log header', () => {
  let grumpkin: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('encrypt and decrypt a log header', () => {
    const ephSecretKey = GrumpkinScalar.random();
    const viewingSecretKey = GrumpkinScalar.random();

    const ephPubKey = grumpkin.mul(Grumpkin.generator, ephSecretKey);
    const viewingPubKey = grumpkin.mul(Grumpkin.generator, viewingSecretKey);

    const header = new EncryptedLogHeader(AztecAddress.random());

    const encrypted = header.computeCiphertext(ephSecretKey, viewingPubKey);

    const recreated = EncryptedLogHeader.fromCiphertext(encrypted, viewingSecretKey, ephPubKey);

    expect(recreated.toBuffer()).toEqual(header.toBuffer());
  });

  it('encrypt a log header, generate input for noir test', () => {
    // The following 2 are arbitrary fixed values - fixed in order to test a match with Noir
    const viewingSecretKey = new GrumpkinScalar(0x23b3127c127b1f29a7adff5cccf8fb06649e7ca01d9de27b21624098b897babdn);
    const ephSecretKey = new GrumpkinScalar(0x1fdd0dd8c99b21af8e00d2d130bdc263b36dadcbea84ac5ec9293a0660deca01n);

    const viewingPubKey = grumpkin.mul(Grumpkin.generator, viewingSecretKey);

    const header = new EncryptedLogHeader(AztecAddress.fromBigInt(BigInt('0xdeadbeef')));

    const encrypted = header.computeCiphertext(ephSecretKey, viewingPubKey);

    const byteArrayString = `[${encrypted
      .toString('hex')
      .match(/.{1,2}/g)!
      .map(byte => parseInt(byte, 16))}]`;

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/encrypted_logs/header.nr',
      'expected_header_ciphertext',
      byteArrayString,
    );
  });
});
