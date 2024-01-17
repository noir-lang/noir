import { open } from 'lmdb';

import { describeArchiverDataStore } from './archiver_store_test_suite.js';
import { LMDBArchiverStore } from './lmdb_archiver_store.js';

describe('LMDB Memory Store', () => {
  let archiverStore: LMDBArchiverStore;

  beforeEach(() => {
    archiverStore = new LMDBArchiverStore(open({} as any));
  });

  describeArchiverDataStore('LMDBArchiverStore', () => archiverStore);
});
