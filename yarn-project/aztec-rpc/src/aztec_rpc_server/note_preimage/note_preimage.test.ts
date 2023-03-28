import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { randomBytes } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { NotePreImage } from './note_preimage.js';

describe('note_preimage', () => {
  it('convert to and from buffer', () => {
    const fields = Array.from({ length: 5 }).map(() => Fr.random());
    const notePreImage = new NotePreImage(fields);
    const buf = notePreImage.toBuffer();
    expect(NotePreImage.fromBuffer(buf)).toEqual(notePreImage);
  });

  it('convert to and from encrypted buffer', async () => {
    const grumpkin = new Grumpkin(await BarretenbergWasm.new());
    const fields = Array.from({ length: 5 }).map(() => Fr.random());
    const notePreImage = new NotePreImage(fields);
    const ownerPrivKey = randomBytes(32);
    const ownerPubKey = AztecAddress.fromBuffer(grumpkin.mul(Grumpkin.generator, ownerPrivKey));
    const ephPrivKey = randomBytes(32);
    const encrypted = notePreImage.toEncryptedBuffer(ownerPubKey, ephPrivKey, grumpkin);
    const decrypted = NotePreImage.fromEncryptedBuffer(encrypted, ownerPrivKey, grumpkin);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted).toEqual(notePreImage);
  });
});
