import { type AztecKVStore } from '@aztec/kv-store';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { Archiver } from './archiver/archiver.js';
import { type ArchiverConfig } from './archiver/config.js';
import { KVArchiverDataStore } from './archiver/index.js';
import { createArchiverClient } from './rpc/archiver_client.js';

export function createArchiver(
  config: ArchiverConfig,
  store: AztecKVStore,
  telemetry: TelemetryClient = new NoopTelemetryClient(),
  opts: { blockUntilSync: boolean } = { blockUntilSync: true },
) {
  if (!config.archiverUrl) {
    // first create and sync the archiver
    const archiverStore = new KVArchiverDataStore(store, config.maxLogs);
    return Archiver.createAndSync(config, archiverStore, telemetry, opts.blockUntilSync);
  } else {
    return createArchiverClient(config.archiverUrl);
  }
}
