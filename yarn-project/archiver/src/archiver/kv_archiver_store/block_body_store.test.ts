import { Body } from '@aztec/circuit-types';
import { openTmpStore } from '@aztec/kv-store/utils';

import { KVArchiverDataStore } from './kv_archiver_store.js';

describe('Block Body Store', () => {
  let archiverStore: KVArchiverDataStore;

  beforeEach(() => {
    archiverStore = new KVArchiverDataStore(openTmpStore());
  });

  it('Should add and return block bodies', async () => {
    const body = Body.random(1);

    await archiverStore.addBlockBodies([body]);

    const txsEffectsHash = body.getTxsEffectsHash();

    const [returnedBody] = await archiverStore.getBlockBodies([txsEffectsHash]);

    expect(body).toStrictEqual(returnedBody);
  });
});
