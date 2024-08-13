import { randomExtendedNote, randomUniqueNote } from '../mocks.js';
import { ExtendedNote, UniqueNote } from './extended_note.js';

describe('Extended Note', () => {
  it('convert to and from buffer', () => {
    const extendedNote = randomExtendedNote();
    const buf = extendedNote.toBuffer();
    expect(ExtendedNote.fromBuffer(buf)).toEqual(extendedNote);
  });
});

describe('Unique Note', () => {
  it('convert to and from buffer', () => {
    const uniqueNote = randomUniqueNote();
    const buf = uniqueNote.toBuffer();
    expect(UniqueNote.fromBuffer(buf)).toEqual(uniqueNote);
  });
});
