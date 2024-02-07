import { openTmpStore } from '@aztec/kv-store/utils';

import { AztecKVTxPool } from './aztec_kv_tx_pool.js';
import { describeTxPool } from './tx_pool_test_suite.js';

describe('In-Memory TX pool', () => {
  let txPool: AztecKVTxPool;
  beforeEach(() => {
    txPool = new AztecKVTxPool(openTmpStore());
  });

  describeTxPool(() => txPool);
});
