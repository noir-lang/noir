import { type L2Block } from '@aztec/circuit-types';
import {
  Attributes,
  type Gauge,
  type Histogram,
  Metrics,
  type TelemetryClient,
  ValueType,
  exponentialBuckets,
} from '@aztec/telemetry-client';

export class ArchiverInstrumentation {
  private blockHeight: Gauge;
  private blockSize: Gauge;
  private syncDuration: Histogram;

  constructor(telemetry: TelemetryClient) {
    const meter = telemetry.getMeter('Archiver');
    this.blockHeight = meter.createGauge(Metrics.ARCHIVER_BLOCK_HEIGHT, {
      description: 'The height of the latest block processed by the archiver',
      valueType: ValueType.INT,
    });

    this.blockSize = meter.createGauge(Metrics.ARCHIVER_BLOCK_SIZE, {
      description: 'The number of transactions in a block',
      valueType: ValueType.INT,
    });

    this.syncDuration = meter.createHistogram(Metrics.ARCHIVER_SYNC_DURATION, {
      unit: 'ms',
      description: 'Duration to sync a block',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: exponentialBuckets(1, 16),
      },
    });
  }

  public processNewBlocks(syncTimePerBlock: number, blocks: L2Block[]) {
    this.syncDuration.record(syncTimePerBlock);
    this.blockHeight.record(Math.max(...blocks.map(b => b.number)));
    for (const block of blocks) {
      this.blockSize.record(block.body.txEffects.length);
    }
  }

  public updateLastProvenBlock(blockNumber: number) {
    this.blockHeight.record(blockNumber, { [Attributes.STATUS]: 'proven' });
  }
}
