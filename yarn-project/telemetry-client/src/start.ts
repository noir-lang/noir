import { createDebugLogger } from '@aztec/foundation/log';

import { NoopTelemetryClient } from './noop.js';
import { OpenTelemetryClient } from './otel.js';
import { type TelemetryClient } from './telemetry.js';

export interface TelemetryClientConfig {
  collectorBaseUrl?: URL;
  serviceName: string;
  serviceVersion: string;
  networkId: string;
}

export function createAndStartTelemetryClient(config: TelemetryClientConfig): TelemetryClient {
  const log = createDebugLogger('aztec:telemetry-client');
  if (config.collectorBaseUrl) {
    log.info('Using OpenTelemetry client');
    return OpenTelemetryClient.createAndStart(
      config.serviceName,
      config.serviceVersion,
      config.networkId,
      config.collectorBaseUrl,
      log,
    );
  } else {
    log.info('Using NoopTelemetryClient');
    return new NoopTelemetryClient();
  }
}

export function getConfigEnvVars(): TelemetryClientConfig {
  const {
    TEL_COLLECTOR_BASE_URL,
    TEL_SERVICE_NAME = 'aztec',
    TEL_SERVICE_VERSION = '0.0.0',
    TEL_NETWORK_ID = 'local',
  } = process.env;

  return {
    collectorBaseUrl: TEL_COLLECTOR_BASE_URL ? new URL(TEL_COLLECTOR_BASE_URL) : undefined,
    serviceName: TEL_SERVICE_NAME,
    serviceVersion: TEL_SERVICE_VERSION,
    networkId: TEL_NETWORK_ID,
  };
}
