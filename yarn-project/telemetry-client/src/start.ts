import { NoopTelemetryClient } from './noop.js';
import { OpenTelemetryClient } from './otel.js';
import { type TelemetryClient } from './telemetry.js';

export interface TelemetryClientConfig {
  collectorBaseUrl?: URL;
}

export function createAndStartTelemetryClient(
  config: TelemetryClientConfig,
  serviceName: string,
  serviceVersion?: string,
): TelemetryClient {
  if (config.collectorBaseUrl) {
    return OpenTelemetryClient.createAndStart(serviceName, serviceVersion ?? '0.0.0', config.collectorBaseUrl);
  } else {
    return new NoopTelemetryClient();
  }
}

export function getConfigEnvVars(): TelemetryClientConfig {
  const { OTEL_COLLECTOR_BASE_URL } = process.env;

  return {
    collectorBaseUrl: OTEL_COLLECTOR_BASE_URL ? new URL(OTEL_COLLECTOR_BASE_URL) : undefined,
  };
}
