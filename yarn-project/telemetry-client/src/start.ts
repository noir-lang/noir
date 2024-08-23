import { createDebugLogger } from '@aztec/foundation/log';

import { type TelemetryClientConfig } from './config.js';
import { NoopTelemetryClient } from './noop.js';
import { OpenTelemetryClient } from './otel.js';
import { type TelemetryClient } from './telemetry.js';

export * from './config.js';

export async function createAndStartTelemetryClient(config: TelemetryClientConfig): Promise<TelemetryClient> {
  const log = createDebugLogger('aztec:telemetry-client');
  if (config.metricsCollectorUrl) {
    log.info('Using OpenTelemetry client');
    return await OpenTelemetryClient.createAndStart(config.metricsCollectorUrl, config.tracesCollectorUrl, log);
  } else {
    log.info('Using NoopTelemetryClient');
    return new NoopTelemetryClient();
  }
}
