import { open } from 'lmdb';

import { LmdbAztecSingleton } from './singleton.js';

describe('LmdbAztecSingleton', () => {
  let singleton: LmdbAztecSingleton<string>;
  beforeEach(() => {
    singleton = new LmdbAztecSingleton(open({} as any), 'test');
  });

  it('returns undefined if the value is not set', () => {
    expect(singleton.get()).toEqual(undefined);
  });

  it('should be able to set and get values', async () => {
    expect(await singleton.set('foo')).toEqual(true);
    expect(singleton.get()).toEqual('foo');
  });

  it('overwrites the value if it is set again', async () => {
    expect(await singleton.set('foo')).toEqual(true);
    expect(await singleton.set('bar')).toEqual(true);
    expect(singleton.get()).toEqual('bar');
  });
});
