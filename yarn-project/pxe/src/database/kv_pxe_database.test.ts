import { AztecLmdbStore } from '@aztec/kv-store';

import { KVPxeDatabase } from './kv_pxe_database.js';
import { describePxeDatabase } from './pxe_database_test_suite.js';

describe('KVPxeDatabase', () => {
  let database: KVPxeDatabase;

  beforeEach(async () => {
    database = new KVPxeDatabase(await AztecLmdbStore.openTmp());
  });

  describePxeDatabase(() => database);
});
