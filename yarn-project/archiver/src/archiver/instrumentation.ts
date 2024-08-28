import { type L2Block } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import {
  Attributes,
  type Gauge,
  type Histogram,
  Metrics,
  type TelemetryClient,
  type UpDownCounter,
  ValueType,
  exponentialBuckets,
  millisecondBuckets,
} from '@aztec/telemetry-client';

export class ArchiverInstrumentation {
  private blockHeight: Gauge;
  private blockSize: Gauge;
  private syncDuration: Histogram;
  private proofsSubmittedDelay: Histogram;
  private proofsSubmittedCount: UpDownCounter;

  private log = createDebugLogger('aztec:archiver:instrumentation');

  constructor(private telemetry: TelemetryClient) {
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

    this.proofsSubmittedCount = meter.createUpDownCounter(Metrics.ARCHIVER_ROLLUP_PROOF_COUNT, {
      description: 'Number of proofs submitted',
      valueType: ValueType.INT,
    });

    this.proofsSubmittedDelay = meter.createHistogram(Metrics.ARCHIVER_ROLLUP_PROOF_DELAY, {
      unit: 'ms',
      description: 'Time after a block is submitted until its proof is published',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: millisecondBuckets(1, 80), // 10ms -> ~3hs
      },
    });
  }

  public isEnabled(): boolean {
    return this.telemetry.isEnabled();
  }

  public processNewBlocks(syncTimePerBlock: number, blocks: L2Block[]) {
    this.syncDuration.record(Math.ceil(syncTimePerBlock));
    this.blockHeight.record(Math.max(...blocks.map(b => b.number)));
    for (const block of blocks) {
      this.blockSize.record(block.body.txEffects.length);
    }
  }

  public updateLastProvenBlock(blockNumber: number) {
    this.blockHeight.record(blockNumber, { [Attributes.STATUS]: 'proven' });
  }

  public processProofsVerified(logs: { proverId: string; l2BlockNumber: bigint; delay: bigint }[]) {
    for (const log of logs) {
      this.log.debug('Recording proof verified event', log);
      this.proofsSubmittedCount.add(1, {
        [Attributes.ROLLUP_PROVER_ID]: log.proverId,
        [Attributes.PROOF_TIMED_OUT]: log.delay > 20n * 60n * 1000n,
      });
      this.proofsSubmittedDelay.record(Math.ceil(Number(log.delay)), {
        [Attributes.ROLLUP_PROVER_ID]: log.proverId,
      });
    }
  }
}
