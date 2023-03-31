import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { AztecAddress, randomBytes } from '@aztec/foundation';
import { Fr, Point } from '@aztec/foundation/fields';
import { NotePreimage } from './note_preimage.js';
import { TxAuxData } from './tx_aux_data.js';

export const randomTxAuxData = () => {
  const fields = Array.from({ length: 5 }).map(() => Fr.random());
  const notePreImage = new NotePreimage(fields);
  const contractAddress = AztecAddress.random();
  const storageSlot = Fr.random();
  return new TxAuxData(notePreImage, contractAddress, storageSlot);
};

describe('tx_aux_data', () => {
  it('convert to and from buffer', () => {
    const txAuxData = randomTxAuxData();
    const buf = txAuxData.toBuffer();
    expect(TxAuxData.fromBuffer(buf)).toEqual(txAuxData);
  });

  it('convert to and from encrypted buffer', async () => {
    const grumpkin = new Grumpkin(await BarretenbergWasm.new());
    const txAuxData = randomTxAuxData();
    const ownerPrivKey = randomBytes(32);
    const ownerPubKey = Point.fromBuffer(grumpkin.mul(Grumpkin.generator, ownerPrivKey));
    const ephPrivKey = randomBytes(32);
    const encrypted = txAuxData.toEncryptedBuffer(ownerPubKey, ephPrivKey, grumpkin);
    const decrypted = TxAuxData.fromEncryptedBuffer(encrypted, ownerPrivKey, grumpkin);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted).toEqual(txAuxData);
  });
});
