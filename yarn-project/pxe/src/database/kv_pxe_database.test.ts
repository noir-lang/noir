import { openTmpStore } from '@aztec/kv-store/utils';

import { KVPxeDatabase } from './kv_pxe_database.js';
import { describePxeDatabase } from './pxe_database_test_suite.js';

describe('KVPxeDatabase', () => {
  let database: KVPxeDatabase;

  beforeEach(() => {
    database = new KVPxeDatabase(openTmpStore());
  });

  describePxeDatabase(() => database);
});
