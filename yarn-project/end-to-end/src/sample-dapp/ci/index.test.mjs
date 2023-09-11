import { waitForSandbox } from '@aztec/aztec.js';

import { deploy } from '../deploy.mjs';
import { main } from '../index.mjs';

// Tests on our CI that all scripts included in the guide work fine
describe('sample-dapp', () => {
  it('deploys and runs without errors', async () => {
    await waitForSandbox();
    await deploy();
    await main();
  }, 60_000);
});
