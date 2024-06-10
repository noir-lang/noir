import { Fr } from '@aztec/foundation/fields';

import { Event, Note } from './payload.js';

describe('note', () => {
  it('convert to and from buffer', () => {
    const fields = Array.from({ length: 5 }).map(() => Fr.random());
    const note = new Note(fields);
    const buf = note.toBuffer();
    expect(Note.fromBuffer(buf)).toEqual(note);
  });
});

describe('event', () => {
  it('convert to and from buffer', () => {
    const fields = Array.from({ length: 5 }).map(() => Fr.random());
    const note = new Event(fields);
    const buf = note.toBuffer();
    expect(Event.fromBuffer(buf)).toEqual(note);
  });
});
