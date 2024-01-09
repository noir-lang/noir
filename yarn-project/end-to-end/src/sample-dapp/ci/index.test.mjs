import { createPXEClient, waitForPXE } from '@aztec/aztec.js';

import { deploy } from '../deploy.mjs';
import { main } from '../index.mjs';

const { PXE_URL = '' } = process.env;

// Tests on our CI that all scripts included in the guide work fine
describe('sample-dapp', () => {
  it('deploys and runs without errors', async () => {
    await waitForPXE(createPXEClient(PXE_URL));
    await deploy();
    await main();
  }, 90_000);
});
