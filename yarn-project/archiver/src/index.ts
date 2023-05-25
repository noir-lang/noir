import { fileURLToPath } from 'url';
import { createPublicClient, http } from 'viem';
import { localhost } from 'viem/chains';
import { Archiver, getConfigEnvVars } from './archiver/index.js';
import { MemoryArchiverStore } from './archiver/archiver_store.js';
import { createLogger } from '@aztec/foundation/log';

export * from './archiver/index.js';

const log = createLogger('aztec:archiver_init');

/**
 * A function which instantiates and starts Archiver.
 */
// eslint-disable-next-line require-await
async function main() {
  const config = getConfigEnvVars();
  const { rpcUrl, rollupContract, inboxContract, unverifiedDataEmitterContract, searchStartBlock } = config;

  const publicClient = createPublicClient({
    chain: localhost,
    transport: http(rpcUrl),
  });

  const archiverStore = new MemoryArchiverStore();

  const archiver = new Archiver(
    publicClient,
    rollupContract,
    inboxContract,
    unverifiedDataEmitterContract,
    searchStartBlock,
    archiverStore,
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
    log(err);
    process.exit(1);
  });
}
