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

    const txsHash = body.getCalldataHash();

    const [returnedBody] = await archiverStore.getBlockBodies([txsHash]);

    expect(body).toStrictEqual(returnedBody);
  });
});
