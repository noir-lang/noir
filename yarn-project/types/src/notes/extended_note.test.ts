import { randomExtendedNote } from '../mocks.js';
import { ExtendedNote } from './extended_note.js';

describe('Extended Note', () => {
  it('convert to and from buffer', () => {
    const extendedNote = randomExtendedNote();
    const buf = extendedNote.toBuffer();
    expect(ExtendedNote.fromBuffer(buf)).toEqual(extendedNote);
  });
});
