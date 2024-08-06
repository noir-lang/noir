import { ProvingRequestType } from '@aztec/circuit-types';
import { Attributes, type Gauge, type Histogram, Metrics, type TelemetryClient } from '@aztec/telemetry-client';

export class ProvingQueueMetrics {
  private jobSize: Histogram;
  private queueSize: Gauge;

  constructor(client: TelemetryClient, name = 'ProvingQueueMetrics') {
    const meter = client.getMeter(name);
    this.jobSize = meter.createHistogram(Metrics.PROVING_QUEUE_JOB_SIZE, {
      description: 'Size of proving job',
      unit: 'by',
    });

    this.queueSize = meter.createGauge(Metrics.PROVING_QUEUE_SIZE, {
      description: 'Size of proving queue',
    });
  }

  recordNewJob(type: ProvingRequestType, size: number) {
    this.jobSize.record(size, {
      [Attributes.PROVING_JOB_TYPE]: ProvingRequestType[type],
    });
  }

  recordQueueSize(size: number) {
    this.queueSize.record(size);
  }
}
