import { EthAddress } from '@aztec/foundation';
import { fileURLToPath } from 'url';
import { createPublicClient, getAddress, http } from 'viem';
import { localhost } from 'viem/chains';
import { Archiver } from './archiver/index.js';

export * from './archiver/index.js';

const {
  ETHEREUM_HOST = 'http://127.0.0.1:8545/',
  ROLLUP_ADDRESS = '0x5FbDB2315678afecb367f032d93F642f64180aa3',
  UNVERIFIED_DATA_EMITTER_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
} = process.env;

/**
 * A function which instantiates and starts Archiver.
 */
// eslint-disable-next-line require-await
async function main() {
  const rollupAddress = getAddress(ROLLUP_ADDRESS);
  const unverifiedDataEmitterAddress = getAddress(UNVERIFIED_DATA_EMITTER_ADDRESS);

  const publicClient = createPublicClient({
    chain: localhost,
    transport: http(ETHEREUM_HOST),
  });

  const archiver = new Archiver(
    publicClient,
    EthAddress.fromString(rollupAddress),
    EthAddress.fromString(unverifiedDataEmitterAddress),
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
    console.log(err);
    process.exit(1);
  });
}
