import { AztecAddress, GrumpkinScalar } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { updateInlineTestData } from '@aztec/foundation/testing';

import { EncryptedLogOutgoingBody } from './encrypted_log_outgoing_body.js';

describe('encrypt log outgoing body', () => {
  let grumpkin: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('encrypt and decrypt a log outgoing body', () => {
    const ephSk = GrumpkinScalar.random();
    const recipientIvsk = GrumpkinScalar.random();
    const senderOvskApp = GrumpkinScalar.random();

    const ephPk = grumpkin.mul(Grumpkin.generator, ephSk);
    const recipientIvpk = grumpkin.mul(Grumpkin.generator, recipientIvsk);

    const recipientAddress = AztecAddress.random();

    const body = new EncryptedLogOutgoingBody(ephSk, recipientAddress, recipientIvpk);

    const encrypted = body.computeCiphertext(senderOvskApp, ephPk);

    const recreated = EncryptedLogOutgoingBody.fromCiphertext(encrypted, senderOvskApp, ephPk);

    expect(recreated.toBuffer()).toEqual(body.toBuffer());
  });

  it('encrypt a log outgoing body, generate input for noir test', () => {
    const ephSk = new GrumpkinScalar(0x0f096b423017226a18461115fa8d34bbd0d302ee245dfaf2807e604eec4715fen);
    const recipientIvsk = new GrumpkinScalar(0x0f4d97c25d578f9348251a71ca17ae314828f8f95676ebb481df163f87fd4022n);
    const senderOvskApp = new GrumpkinScalar(0x089c6887cb1446d86c64e81afc78048b74d2e28c6bc5176ac02cf7c7d36a444en);

    const ephPk = grumpkin.mul(Grumpkin.generator, ephSk);
    const recipientIvpk = grumpkin.mul(Grumpkin.generator, recipientIvsk);

    const recipientAddress = AztecAddress.fromBigInt(BigInt('0xdeadbeef'));

    const body = new EncryptedLogOutgoingBody(ephSk, recipientAddress, recipientIvpk);

    const encrypted = body.computeCiphertext(senderOvskApp, ephPk);

    const recreated = EncryptedLogOutgoingBody.fromCiphertext(encrypted, senderOvskApp, ephPk);

    expect(recreated.toBuffer()).toEqual(body.toBuffer());

    const byteArrayString = `[${encrypted
      .toString('hex')
      .match(/.{1,2}/g)!
      .map(byte => parseInt(byte, 16))}]`;

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/encrypted_logs/outgoing_body.nr',
      'expected_outgoing_body_ciphertext',
      byteArrayString,
    );
  });
});
