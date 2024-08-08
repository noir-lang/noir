import { createDebugLogger } from '@aztec/foundation/log';

import { type TelemetryClientConfig } from './config.js';
import { NoopTelemetryClient } from './noop.js';
import { OpenTelemetryClient } from './otel.js';
import { type TelemetryClient } from './telemetry.js';

export * from './config.js';

export function createAndStartTelemetryClient(config: TelemetryClientConfig): TelemetryClient {
  const log = createDebugLogger('aztec:telemetry-client');
  if (config.collectorBaseUrl) {
    log.info('Using OpenTelemetry client');
    return OpenTelemetryClient.createAndStart(config.collectorBaseUrl, log);
  } else {
    log.info('Using NoopTelemetryClient');
    return new NoopTelemetryClient();
  }
}
