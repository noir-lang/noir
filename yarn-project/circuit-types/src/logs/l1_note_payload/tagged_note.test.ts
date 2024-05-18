import { AztecAddress } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { GrumpkinScalar } from '@aztec/foundation/fields';

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

  describe('encrypt and decrypt a full log', () => {
    let ovsk: GrumpkinScalar;
    let ivsk: GrumpkinScalar;

    let taggedNote: TaggedNote;
    let encrypted: Buffer;

    beforeAll(() => {
      ovsk = GrumpkinScalar.random();
      ivsk = GrumpkinScalar.random();

      const ephSk = GrumpkinScalar.random();

      const recipientAddress = AztecAddress.random();
      const ivpk = grumpkin.mul(Grumpkin.generator, ivsk);

      const payload = L1NotePayload.random();

      taggedNote = new TaggedNote(payload);

      encrypted = taggedNote.encrypt(ephSk, recipientAddress, ivpk, ovsk);
    });

    it('decrypt a log as incoming', () => {
      const recreated = TaggedNote.decryptAsIncoming(encrypted, ivsk);

      expect(recreated?.toBuffer()).toEqual(taggedNote.toBuffer());
    });

    it('decrypt a log as outgoing', () => {
      const recreated = TaggedNote.decryptAsOutgoing(encrypted, ovsk);

      expect(recreated?.toBuffer()).toEqual(taggedNote.toBuffer());
    });
  });
});
