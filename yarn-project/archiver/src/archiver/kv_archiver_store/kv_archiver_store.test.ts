import { openTmpStore } from '@aztec/kv-store/utils';

import { describeArchiverDataStore } from '../archiver_store_test_suite.js';
import { KVArchiverDataStore } from './kv_archiver_store.js';

describe('KVArchiverDataStore', () => {
  let archiverStore: KVArchiverDataStore;

  beforeEach(() => {
    archiverStore = new KVArchiverDataStore(openTmpStore());
  });

  describeArchiverDataStore('ArchiverStore', () => archiverStore);
});
