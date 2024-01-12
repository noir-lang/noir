import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { GrumpkinScalar, Point } from '@aztec/foundation/fields';

import { L1NotePayload } from './l1_note_payload.js';

describe('L1 Note Payload', () => {
  let grumpkin: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('convert to and from buffer', () => {
    const payload = L1NotePayload.random();
    const buf = payload.toBuffer();
    expect(L1NotePayload.fromBuffer(buf)).toEqual(payload);
  });

  it('convert to and from encrypted buffer', () => {
    const payload = L1NotePayload.random();
    const ownerPrivKey = GrumpkinScalar.random();
    const ownerPubKey = grumpkin.mul(Grumpkin.generator, ownerPrivKey);
    const encrypted = payload.toEncryptedBuffer(ownerPubKey, grumpkin);
    const decrypted = L1NotePayload.fromEncryptedBuffer(encrypted, ownerPrivKey, grumpkin);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted).toEqual(payload);
  });

  it('return undefined if unable to decrypt the encrypted buffer', () => {
    const payload = L1NotePayload.random();
    const ownerPubKey = Point.random();
    const encrypted = payload.toEncryptedBuffer(ownerPubKey, grumpkin);
    const randomPrivKey = GrumpkinScalar.random();
    const decrypted = L1NotePayload.fromEncryptedBuffer(encrypted, randomPrivKey, grumpkin);
    expect(decrypted).toBeUndefined();
  });
});
