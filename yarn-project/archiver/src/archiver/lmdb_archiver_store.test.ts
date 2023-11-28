import { mkdtemp, rm } from 'fs/promises';
import { RootDatabase, open } from 'lmdb';
import { tmpdir } from 'os';
import { join } from 'path';

import { describeArchiverDataStore } from './archiver_store_test_suite.js';
import { LMDBArchiverStore } from './lmdb_archiver_store.js';

describe('LMDB Memory Store', () => {
  let archiverStore: LMDBArchiverStore;
  let tmpDbLocation: string;
  let tmpDb: RootDatabase;

  beforeAll(async () => {
    tmpDbLocation = await mkdtemp(join(tmpdir(), 'archiver-store-test-'));
    tmpDb = open(tmpDbLocation, {});
  });

  afterAll(async () => {
    await tmpDb.close();
    await rm(tmpDbLocation, { recursive: true });
  });

  beforeEach(() => {
    archiverStore = new LMDBArchiverStore(tmpDb);
  });

  afterEach(async () => {
    await archiverStore?.close();
    await tmpDb.clearAsync();
  });

  describeArchiverDataStore('LMDBArchiverStore', () => archiverStore);
});
