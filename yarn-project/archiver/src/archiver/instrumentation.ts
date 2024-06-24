import { type L2Block } from '@aztec/circuit-types';
import { type Gauge, type Histogram, Metrics, type TelemetryClient, ValueType } from '@aztec/telemetry-client';

export class ArchiverInstrumentation {
  private blockHeight: Gauge;
  private blockSize: Histogram;

  constructor(telemetry: TelemetryClient) {
    const meter = telemetry.getMeter('Archiver');
    this.blockHeight = meter.createGauge(Metrics.ARCHIVER_BLOCK_HEIGHT, {
      description: 'The height of the latest block processed by the archiver',
      valueType: ValueType.INT,
    });

    this.blockSize = meter.createHistogram(Metrics.ARCHIVER_BLOCK_SIZE, {
      description: 'The number of transactions processed per block',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192],
      },
    });
  }

  public processNewBlocks(blocks: L2Block[]) {
    this.blockHeight.record(Math.max(...blocks.map(b => b.number)));
    for (const block of blocks) {
      this.blockSize.record(block.body.txEffects.length);
    }
  }
}
