import { type ConfigMappingsType, getConfigFromMappings } from '@aztec/foundation/config';

export interface TelemetryClientConfig {
  collectorBaseUrl?: URL;
  serviceName: string;
  networkName: string;
}

export const telemetryClientConfigMappings: ConfigMappingsType<TelemetryClientConfig> = {
  collectorBaseUrl: {
    env: 'OTEL_EXPORTER_OTLP_ENDPOINT',
    description: 'The URL of the telemetry collector',
    parseEnv: (val: string) => new URL(val),
  },
  serviceName: {
    env: 'OTEL_SERVICE_NAME',
    description: 'The URL of the telemetry collector',
    defaultValue: 'aztec',
  },
  networkName: {
    env: 'NETWORK_NAME',
    description: 'The network ID of the telemetry service',
    defaultValue: 'local',
  },
};

export function getConfigEnvVars(): TelemetryClientConfig {
  return getConfigFromMappings<TelemetryClientConfig>(telemetryClientConfigMappings);
}
