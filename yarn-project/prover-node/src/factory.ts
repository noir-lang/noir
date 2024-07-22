import { type Archiver, createArchiver } from '@aztec/archiver';
import { type AztecNode } from '@aztec/circuit-types';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { createStore } from '@aztec/kv-store/utils';
import { createProverClient } from '@aztec/prover-client';
import { getL1Publisher } from '@aztec/sequencer-client';
import { PublicProcessorFactory, createSimulationProvider } from '@aztec/simulator';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';
import { createWorldStateSynchronizer } from '@aztec/world-state';

import { type ProverNodeConfig } from './config.js';
import { ProverNode } from './prover-node.js';
import { AztecNodeTxProvider } from './tx-provider/aztec-node-tx-provider.js';
import { createTxProvider } from './tx-provider/factory.js';

/** Creates a new prover node given a config. */
export async function createProverNode(
  config: ProverNodeConfig,
  deps: {
    telemetry?: TelemetryClient;
    log?: DebugLogger;
    storeLog?: DebugLogger;
    aztecNodeTxProvider?: AztecNode;
    archiver?: Archiver;
  } = {},
) {
  const telemetry = deps.telemetry ?? new NoopTelemetryClient();
  const log = deps.log ?? createDebugLogger('aztec:prover');
  const storeLog = deps.storeLog ?? createDebugLogger('aztec:prover:lmdb');

  const store = await createStore(config, config.l1Contracts.rollupAddress, storeLog);

  const archiver = deps.archiver ?? (await createArchiver(config, store, telemetry, { blockUntilSync: true }));
  log.verbose(`Created archiver and synced to block ${await archiver.getBlockNumber()}`);

  const worldStateConfig = { ...config, worldStateProvenBlocksOnly: true };
  const worldStateSynchronizer = await createWorldStateSynchronizer(worldStateConfig, store, archiver);
  await worldStateSynchronizer.start();

  const simulationProvider = await createSimulationProvider(config, log);

  const prover = await createProverClient(config, worldStateSynchronizer, archiver);

  // REFACTOR: Move publisher out of sequencer package and into an L1-related package
  const publisher = getL1Publisher(config);

  const latestWorldState = worldStateSynchronizer.getLatest();
  const publicProcessorFactory = new PublicProcessorFactory(latestWorldState, archiver, simulationProvider, telemetry);

  const txProvider = deps.aztecNodeTxProvider
    ? new AztecNodeTxProvider(deps.aztecNodeTxProvider)
    : createTxProvider(config);

  return new ProverNode(prover!, publicProcessorFactory, publisher, archiver, txProvider);
}
