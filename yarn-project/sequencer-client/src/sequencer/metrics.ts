import {
  Attributes,
  type Histogram,
  Metrics,
  type TelemetryClient,
  type Tracer,
  type UpDownCounter,
  ValueType,
  millisecondBuckets,
} from '@aztec/telemetry-client';

export class SequencerMetrics {
  public readonly tracer: Tracer;

  private cancelledBlockCounter: UpDownCounter;
  private blocksBuiltCounter: UpDownCounter;
  private blockBuildDuration: Histogram;
  private blockTxCount: Histogram;

  constructor(client: TelemetryClient, name = 'Sequencer') {
    const meter = client.getMeter(name);
    this.tracer = client.getTracer(name);

    this.cancelledBlockCounter = meter.createUpDownCounter(Metrics.SEQUENCER_BLOCK_BUILD_CANCELLED_COUNT);
    this.blocksBuiltCounter = meter.createUpDownCounter(Metrics.SEQUENCER_BLOCK_BUILD_COUNT);
    this.blockBuildDuration = meter.createHistogram(Metrics.SEQUENCER_BLOCK_BUILD_DURATION, {
      unit: 'ms',
      description: 'Duration to build a block',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: millisecondBuckets(2),
      },
    });

    this.blockTxCount = meter.createHistogram(Metrics.SEQUENCER_BLOCK_BUILD_TX_COUNT, {
      description: 'Number of transactions in a block',
      valueType: ValueType.INT,
    });
  }

  recordCancelledBlock() {
    this.cancelledBlockCounter.add(1);
  }

  recordPublishedBlock(buildDurationMs: number) {
    this.blocksBuiltCounter.add(1, {
      [Attributes.OK]: true,
    });
    this.blockBuildDuration.record(Math.ceil(buildDurationMs));
  }

  recordFailedBlock() {
    this.blocksBuiltCounter.add(1, {
      [Attributes.OK]: false,
    });
  }

  recordNewBlock(txCount: number) {
    this.blockTxCount.record(txCount);
  }
}
