import { type TelemetryClient } from '@aztec/telemetry-client';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { type ProverClientConfig } from '../config.js';
import { TxProver } from './tx-prover.js';

export function createProverClient(config: ProverClientConfig, telemetry: TelemetryClient = new NoopTelemetryClient()) {
  return config.disableProver ? undefined : TxProver.new(config, telemetry);
}
