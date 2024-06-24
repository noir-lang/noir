import { type Meter, createNoopMeter } from '@opentelemetry/api';

import { type TelemetryClient } from './telemetry.js';

export class NoopTelemetryClient implements TelemetryClient {
  getMeter(): Meter {
    return createNoopMeter();
  }

  stop(): Promise<void> {
    return Promise.resolve();
  }
}
