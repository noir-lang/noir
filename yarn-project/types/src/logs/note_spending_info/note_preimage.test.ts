import { Fr } from '@aztec/foundation/fields';

import { NotePreimage } from './note_preimage.js';

describe('note_preimage', () => {
  it('convert to and from buffer', () => {
    const fields = Array.from({ length: 5 }).map(() => Fr.random());
    const notePreimage = new NotePreimage(fields);
    const buf = notePreimage.toBuffer();
    expect(NotePreimage.fromBuffer(buf)).toEqual(notePreimage);
  });
});
