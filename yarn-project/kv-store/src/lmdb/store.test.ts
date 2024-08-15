import { mkdtemp } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';

import { AztecLmdbStore } from './store.js';

describe('AztecLmdbStore', () => {
  const itForks = async (store: AztecLmdbStore) => {
    const singleton = store.openSingleton('singleton');
    await singleton.set('foo');

    const forkedStore = await store.fork();
    const forkedSingleton = forkedStore.openSingleton('singleton');
    expect(forkedSingleton.get()).toEqual('foo');
    await forkedSingleton.set('bar');
    expect(singleton.get()).toEqual('foo');
    expect(forkedSingleton.get()).toEqual('bar');
    await forkedSingleton.delete();
    expect(singleton.get()).toEqual('foo');
  };

  it('forks a persistent store', async () => {
    const path = await mkdtemp(join(tmpdir(), 'aztec-store-test-'));
    const store = AztecLmdbStore.open(path, false);
    await itForks(store);
  });

  it('forks a persistent store with no path', async () => {
    const store = AztecLmdbStore.open(undefined, false);
    await itForks(store);
  });

  it('forks an ephemeral store', async () => {
    const store = AztecLmdbStore.open(undefined, true);
    await itForks(store);
  });
});
