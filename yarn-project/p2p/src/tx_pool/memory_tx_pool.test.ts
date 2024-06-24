import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { InMemoryTxPool } from './index.js';
import { describeTxPool } from './tx_pool_test_suite.js';

describe('In-Memory TX pool', () => {
  let inMemoryTxPool: InMemoryTxPool;
  beforeEach(() => {
    inMemoryTxPool = new InMemoryTxPool(new NoopTelemetryClient());
  });

  describeTxPool(() => inMemoryTxPool);
});
