import { type Timer } from '@aztec/foundation/timer';
import { type Histogram, Metrics, type TelemetryClient, ValueType, millisecondBuckets } from '@aztec/telemetry-client';

export class ProverNodeMetrics {
  provingJobDuration: Histogram;

  constructor(client: TelemetryClient, name = 'ProverNode') {
    const meter = client.getMeter(name);
    this.provingJobDuration = meter.createHistogram(Metrics.PROVER_NODE_JOB_DURATION, {
      description: 'Duration of proving job',
      unit: 'ms',
      valueType: ValueType.INT,
      advice: {
        explicitBucketBoundaries: millisecondBuckets(2), // 60 buckets spanning an interval of ~100ms to ~1hour
      },
    });
  }

  public recordProvingJob(timerOrMs: Timer | number) {
    const ms = Math.ceil(typeof timerOrMs === 'number' ? timerOrMs : timerOrMs.ms());
    this.provingJobDuration.record(ms);
  }
}
