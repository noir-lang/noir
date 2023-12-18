import { Database, open } from 'lmdb';

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

    expect(await map.delete('foo')).toEqual(true);

    expect(map.get('foo')).toEqual(undefined);
    expect(map.get('baz')).toEqual('qux');
  });

  it('should be able to iterate over entries', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'qux');

    expect([...map.entries()]).toEqual(
      expect.arrayContaining([
        ['foo', 'bar'],
        ['baz', 'qux'],
      ]),
    );
  });

  it('should be able to iterate over values', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'qux');

    expect([...map.values()]).toEqual(expect.arrayContaining(['bar', 'qux']));
  });

  it('should be able to iterate over keys', async () => {
    await map.set('foo', 'bar');
    await map.set('baz', 'qux');

    expect([...map.keys()]).toEqual(expect.arrayContaining(['foo', 'baz']));
  });

  it('should be able to get multiple values for a single key', async () => {
    await map.set('foo', 'bar');
    await map.set('foo', 'baz');

    expect([...map.getValues('foo')]).toEqual(['bar', 'baz']);
  });
});
