import { type Database, open } from 'lmdb';

import { LmdbAztecSet } from './set.js';

describe('LmdbAztecSet', () => {
  let db: Database;
  let set: LmdbAztecSet<string>;

  beforeEach(() => {
    db = open({ dupSort: true } as any);
    set = new LmdbAztecSet(db, 'test');
  });

  it('should be able to set and get values', async () => {
    await set.add('foo');
    await set.add('baz');

    expect(set.has('foo')).toEqual(true);
    expect(set.has('baz')).toEqual(true);
    expect(set.has('bar')).toEqual(false);
  });

  it('should be able to delete values', async () => {
    await set.add('foo');
    await set.add('baz');

    await set.delete('foo');

    expect(set.has('foo')).toEqual(false);
    expect(set.has('baz')).toEqual(true);
  });

  it('should be able to iterate over entries', async () => {
    await set.add('baz');
    await set.add('foo');

    expect([...set.entries()]).toEqual(['baz', 'foo']);
  });

  it('supports range queries', async () => {
    await set.add('a');
    await set.add('b');
    await set.add('c');
    await set.add('d');

    expect([...set.entries({ start: 'b', end: 'c' })]).toEqual(['b']);
    expect([...set.entries({ start: 'b' })]).toEqual(['b', 'c', 'd']);
    expect([...set.entries({ end: 'c' })]).toEqual(['a', 'b']);
    expect([...set.entries({ start: 'b', end: 'c', reverse: true })]).toEqual(['c']);
    expect([...set.entries({ start: 'b', limit: 1 })]).toEqual(['b']);
    expect([...set.entries({ start: 'b', reverse: true })]).toEqual(['d', 'c']);
    expect([...set.entries({ end: 'b', reverse: true })]).toEqual(['b', 'a']);
  });
});
