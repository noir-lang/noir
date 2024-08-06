import { type Tx } from '@aztec/circuit-types';
import { Attributes, type Histogram, Metrics, type TelemetryClient, type UpDownCounter } from '@aztec/telemetry-client';

export type TxStatus = 'pending' | 'mined';

/**
 * Instrumentation class for the TxPool.
 */
export class TxPoolInstrumentation {
  /** The number of txs in the mempool */
  private txInMempool: UpDownCounter;
  /** Tracks tx size */
  private txSize: Histogram;

  constructor(telemetry: TelemetryClient, name: string) {
    const meter = telemetry.getMeter(name);
    this.txInMempool = meter.createUpDownCounter(Metrics.MEMPOOL_TX_COUNT, {
      description: 'The current number of transactions in the mempool',
    });

    this.txSize = meter.createHistogram(Metrics.MEMPOOL_TX_SIZE, {
      unit: 'By',
      description: 'The size of transactions in the mempool',
      advice: {
        explicitBucketBoundaries: [
          5_000, // 5KB
          10_000,
          20_000,
          50_000,
          75_000,
          100_000, // 100KB
          200_000,
        ],
      },
    });
  }

  public recordTxSize(tx: Tx) {
    this.txSize.record(tx.getSize());
  }

  /**
   * Updates the metrics with the new transactions.
   * @param txs - The transactions to record
   */
  public recordAddedTxs(status: string, count = 1) {
    if (count < 0) {
      throw new Error('Count must be positive');
    }
    if (count === 0) {
      return;
    }
    this.txInMempool.add(count, {
      [Attributes.STATUS]: status,
    });
  }

  /**
   * Updates the metrics by removing transactions from the mempool.
   * @param count - The number of transactions to remove from the mempool
   */
  public recordRemovedTxs(status: string, count = 1) {
    if (count < 0) {
      throw new Error('Count must be positive');
    }
    if (count === 0) {
      return;
    }
    this.txInMempool.add(-1 * count, {
      [Attributes.STATUS]: status,
    });
  }
}
