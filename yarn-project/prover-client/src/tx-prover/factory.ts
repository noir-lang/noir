import { type L2BlockSource } from '@aztec/circuit-types';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';
import { type WorldStateSynchronizer } from '@aztec/world-state';

import { type ProverClientConfig } from '../config.js';
import { TxProver } from './tx-prover.js';

export function createProverClient(
  config: ProverClientConfig,
  worldStateSynchronizer: WorldStateSynchronizer,
  blockSource: L2BlockSource,
  telemetry: TelemetryClient = new NoopTelemetryClient(),
) {
  return config.disableProver ? undefined : TxProver.new(config, worldStateSynchronizer, blockSource, telemetry);
}
