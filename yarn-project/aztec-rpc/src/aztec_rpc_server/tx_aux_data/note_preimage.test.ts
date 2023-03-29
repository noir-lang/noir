import { Fr } from '@aztec/foundation/fields';
import { NotePreimage } from './note_preimage.js';

describe('note_preimage', () => {
  it('convert to and from buffer', () => {
    const fields = Array.from({ length: 5 }).map(() => Fr.random());
    const notePreImage = new NotePreimage(fields);
    const buf = notePreImage.toBuffer();
    expect(NotePreimage.fromBuffer(buf)).toEqual(notePreImage);
  });
});
