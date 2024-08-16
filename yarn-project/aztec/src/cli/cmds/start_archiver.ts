import {
  Archiver,
  type ArchiverConfig,
  KVArchiverDataStore,
  archiverConfigMappings,
  createArchiverRpcServer,
} from '@aztec/archiver';
import { createDebugLogger } from '@aztec/aztec.js';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { createStore } from '@aztec/kv-store/utils';
import {
  createAndStartTelemetryClient,
  getConfigEnvVars as getTelemetryClientConfig,
} from '@aztec/telemetry-client/start';

import { extractRelevantOptions } from '../util.js';

export const startArchiver = async (options: any, signalHandlers: (() => Promise<void>)[]) => {
  const services: ServerList = [];
  // Start a standalone archiver.
  const archiverConfig = extractRelevantOptions<ArchiverConfig>(options, archiverConfigMappings);

  const storeLog = createDebugLogger('aztec:archiver:lmdb');
  const rollupAddress = archiverConfig.l1Contracts.rollupAddress;
  const store = await createStore(archiverConfig, rollupAddress, storeLog);
  const archiverStore = new KVArchiverDataStore(store, archiverConfig.maxLogs);

  const telemetry = createAndStartTelemetryClient(getTelemetryClientConfig());
  const archiver = await Archiver.createAndSync(archiverConfig, archiverStore, telemetry, true);
  const archiverServer = createArchiverRpcServer(archiver);
  services.push({ archiver: archiverServer });
  signalHandlers.push(archiver.stop);
  return services;
};
