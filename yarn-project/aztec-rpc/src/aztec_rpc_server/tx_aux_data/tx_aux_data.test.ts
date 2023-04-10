import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { randomBytes } from '@aztec/foundation';
import { Point } from '@aztec/foundation/fields';
import { TxAuxData } from './tx_aux_data.js';

describe('tx_aux_data', () => {
  let grumpkin: Grumpkin;

  beforeAll(async () => {
    grumpkin = new Grumpkin(await BarretenbergWasm.get());
  });

  it('convert to and from buffer', () => {
    const txAuxData = TxAuxData.random();
    const buf = txAuxData.toBuffer();
    expect(TxAuxData.fromBuffer(buf)).toEqual(txAuxData);
  });

  it('convert to and from encrypted buffer', () => {
    const txAuxData = TxAuxData.random();
    const ownerPrivKey = randomBytes(32);
    const ownerPubKey = Point.fromBuffer(grumpkin.mul(Grumpkin.generator, ownerPrivKey));
    const encrypted = txAuxData.toEncryptedBuffer(ownerPubKey, grumpkin);
    const decrypted = TxAuxData.fromEncryptedBuffer(encrypted, ownerPrivKey, grumpkin);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted).toEqual(txAuxData);
  });

  it('return undefined if unable to decrypt the encrypted buffer', () => {
    const txAuxData = TxAuxData.random();
    const ownerPubKey = Point.random();
    const encrypted = txAuxData.toEncryptedBuffer(ownerPubKey, grumpkin);
    const randomPrivKey = randomBytes(32);
    const decrypted = TxAuxData.fromEncryptedBuffer(encrypted, randomPrivKey, grumpkin);
    expect(decrypted).toBeUndefined();
  });
});
