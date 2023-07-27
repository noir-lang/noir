import { CircuitsWasm, PrivateKey } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { Point } from '@aztec/foundation/fields';

import { NoteSpendingInfo } from './note_spending_info.js';

describe('note_spending_info', () => {
  let grumpkin: Grumpkin;

  beforeAll(async () => {
    grumpkin = new Grumpkin(await CircuitsWasm.get());
  });

  it('convert to and from buffer', () => {
    const noteSpendingInfo = NoteSpendingInfo.random();
    const buf = noteSpendingInfo.toBuffer();
    expect(NoteSpendingInfo.fromBuffer(buf)).toEqual(noteSpendingInfo);
  });

  it('convert to and from encrypted buffer', () => {
    const noteSpendingInfo = NoteSpendingInfo.random();
    const ownerPrivKey = PrivateKey.random();
    const ownerPubKey = grumpkin.mul(Grumpkin.generator, ownerPrivKey);
    const encrypted = noteSpendingInfo.toEncryptedBuffer(ownerPubKey, grumpkin);
    const decrypted = NoteSpendingInfo.fromEncryptedBuffer(encrypted, ownerPrivKey, grumpkin);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted).toEqual(noteSpendingInfo);
  });

  it('return undefined if unable to decrypt the encrypted buffer', () => {
    const noteSpendingInfo = NoteSpendingInfo.random();
    const ownerPubKey = Point.random();
    const encrypted = noteSpendingInfo.toEncryptedBuffer(ownerPubKey, grumpkin);
    const randomPrivKey = PrivateKey.random();
    const decrypted = NoteSpendingInfo.fromEncryptedBuffer(encrypted, randomPrivKey, grumpkin);
    expect(decrypted).toBeUndefined();
  });
});
