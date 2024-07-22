import { createDebugLogger } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { createPublicClient, http } from 'viem';
import { localhost } from 'viem/chains';

import { Archiver, getArchiverConfigFromEnv } from './archiver/index.js';
import { MemoryArchiverStore } from './archiver/memory_archiver_store/memory_archiver_store.js';

export * from './archiver/index.js';
export * from './rpc/index.js';
export * from './factory.js';

const log = createDebugLogger('aztec:archiver');

/**
 * A function which instantiates and starts Archiver.
 */
// eslint-disable-next-line require-await
async function main() {
  const config = getArchiverConfigFromEnv();
  const { rpcUrl, l1Contracts } = config;

  const publicClient = createPublicClient({
    chain: localhost,
    transport: http(rpcUrl),
  });

  const archiverStore = new MemoryArchiverStore(1000);

  const archiver = new Archiver(
    publicClient,
    l1Contracts.rollupAddress,
    l1Contracts.availabilityOracleAddress,
    l1Contracts.inboxAddress,
    l1Contracts.registryAddress,
    archiverStore,
    1000,
    new NoopTelemetryClient(),
  );

  const shutdown = async () => {
    await archiver.stop();
    process.exit(0);
  };
  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);
}

// See https://twitter.com/Rich_Harris/status/1355289863130673153
if (process.argv[1] === fileURLToPath(import.meta.url).replace(/\/index\.js$/, '')) {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  main().catch(err => {
    log.error(err);
    process.exit(1);
  });
}
