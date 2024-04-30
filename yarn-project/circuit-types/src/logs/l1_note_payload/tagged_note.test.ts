import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { GrumpkinScalar, Point } from '@aztec/foundation/fields';

import { L1NotePayload } from './l1_note_payload.js';
import { TaggedNote } from './tagged_note.js';

describe('L1 Note Payload', () => {
  let grumpkin: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('convert to and from buffer', () => {
    const payload = L1NotePayload.random();
    const taggedNote = new TaggedNote(payload);
    const buf = taggedNote.toBuffer();
    expect(TaggedNote.fromBuffer(buf).notePayload).toEqual(taggedNote.notePayload);
  });

  it('convert to and from encrypted buffer', () => {
    const payload = L1NotePayload.random();
    const taggedNote = new TaggedNote(payload);
    const ownerPrivKey = GrumpkinScalar.random();
    const ownerPubKey = grumpkin.mul(Grumpkin.generator, ownerPrivKey);
    const encrypted = taggedNote.toEncryptedBuffer(ownerPubKey);
    const decrypted = TaggedNote.fromEncryptedBuffer(encrypted, ownerPrivKey);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted?.notePayload).toEqual(payload);
  });

  it('return undefined if unable to decrypt the encrypted buffer', () => {
    const payload = L1NotePayload.random();
    const taggedNote = new TaggedNote(payload);
    const ownerPubKey = Point.random();
    const encrypted = taggedNote.toEncryptedBuffer(ownerPubKey);
    const randomPrivKey = GrumpkinScalar.random();
    const decrypted = TaggedNote.fromEncryptedBuffer(encrypted, randomPrivKey);
    expect(decrypted).toBeUndefined();
  });
});
