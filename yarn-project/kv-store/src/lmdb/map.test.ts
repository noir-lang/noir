import { type Database, open } from 'lmdb';

import { LmdbAztecMap } from './map.js';

describe('LmdbAztecMap', () => {
  let db: Database;
  let map: LmdbAztecMap<string, string>;

  beforeEach(() => {
    db = open({ dupSort: true } as any);
    map = new LmdbAztecMap(db, 'test');
  });

  it('should be able to set and get values', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'qux');

    expect(map.get('foo')).toEqual('bar');
    expect(map.get('baz')).toEqual('qux');
    expect(map.get('quux')).toEqual(undefined);
  });

  it('should be able to set values if they do not exist', async () => {
    expect(await map.setIfNotExists('foo', 'bar')).toEqual(true);
    expect(await map.setIfNotExists('foo', 'baz')).toEqual(false);

    expect(map.get('foo')).toEqual('bar');
  });

  it('should be able to delete values', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'qux');

    await map.delete('foo');

    expect(map.get('foo')).toEqual(undefined);
    expect(map.get('baz')).toEqual('qux');
  });

  it('should be able to iterate over entries', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'qux');

    expect([...map.entries()]).toEqual([
      ['baz', 'qux'],
      ['foo', 'bar'],
    ]);
  });

  it('should be able to iterate over values', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'quux');

    expect([...map.values()]).toEqual(['quux', 'bar']);
  });

  it('should be able to iterate over keys', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'qux');

    expect([...map.keys()]).toEqual(['baz', 'foo']);
  });

  it('should be able to get multiple values for a single key', async () => {
    await map.set('foo', 'bar');
    await map.set('foo', 'baz');

    expect([...map.getValues('foo')]).toEqual(['bar', 'baz']);
  });

  it('supports tuple keys', async () => {
    const map = new LmdbAztecMap<[number, string], string>(db, 'test');

    await map.set([5, 'bar'], 'val');
    await map.set([0, 'foo'], 'val');

    expect([...map.keys()]).toEqual([
      [0, 'foo'],
      [5, 'bar'],
    ]);

    expect(map.get([5, 'bar'])).toEqual('val');
  });

  it('supports range queries', async () => {
    await map.set('a', 'a');
    await map.set('b', 'b');
    await map.set('c', 'c');
    await map.set('d', 'd');

    expect([...map.keys({ start: 'b', end: 'c' })]).toEqual(['b']);
    expect([...map.keys({ start: 'b' })]).toEqual(['b', 'c', 'd']);
    expect([...map.keys({ end: 'c' })]).toEqual(['a', 'b']);
    expect([...map.keys({ start: 'b', end: 'c', reverse: true })]).toEqual(['c']);
    expect([...map.keys({ start: 'b', limit: 1 })]).toEqual(['b']);
    expect([...map.keys({ start: 'b', reverse: true })]).toEqual(['d', 'c']);
    expect([...map.keys({ end: 'b', reverse: true })]).toEqual(['b', 'a']);
  });
});
