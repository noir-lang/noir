import { AztecLmdbStore } from '@aztec/kv-store';

import { AztecKVTxPool } from './aztec_kv_tx_pool.js';
import { describeTxPool } from './tx_pool_test_suite.js';

describe('In-Memory TX pool', () => {
  let txPool: AztecKVTxPool;
  beforeEach(async () => {
    txPool = new AztecKVTxPool(await AztecLmdbStore.openTmp());
  });

  describeTxPool(() => txPool);
});
