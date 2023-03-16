import { createPublicClient, getAddress, http } from 'viem';
import { localhost } from 'viem/chains';
import { Archiver } from './archiver.js';

const {
  ETHEREUM_HOST = 'http://localhost:8545/',
  ROLLUP_ADDRESS = '0x5FbDB2315678afecb367f032d93F642f64180aa3',
  YEETER_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
} = process.env;

/**
 * A function which instantiates and starts Archiver.
 */
async function main() {
  const rollupAddress = getAddress(ROLLUP_ADDRESS);
  const yeeterAddress = getAddress(YEETER_ADDRESS);

  const publicClient = createPublicClient({
    chain: localhost,
    transport: http(ETHEREUM_HOST),
  });

  const archiver = new Archiver(publicClient, rollupAddress, yeeterAddress);

  const shutdown = () => {
    archiver.stop();
    process.exit(0);
  };
  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);

  await archiver.start();
}

main().catch(err => {
  console.log(err);
  process.exit(1);
});
