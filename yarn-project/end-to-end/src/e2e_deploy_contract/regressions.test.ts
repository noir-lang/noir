import { getDeployedTestAccountsWallets } from '@aztec/accounts/testing';
import { createPXEClient, makeFetch } from '@aztec/aztec.js';
import { StatefulTestContract } from '@aztec/noir-contracts.js';

import { DeployTest } from './deploy_test.js';

describe('e2e_deploy_contract regressions', () => {
  const t = new DeployTest('regressions');

  beforeAll(async () => {
    await t.setup();
  });

  afterAll(() => t.teardown());

  it('fails properly when trying to deploy a contract with a failing constructor with a pxe client with retries', async () => {
    const { PXE_URL } = process.env;
    if (!PXE_URL) {
      return;
    }
    const pxeClient = createPXEClient(PXE_URL, makeFetch([1, 2, 3], false));
    const [wallet] = await getDeployedTestAccountsWallets(pxeClient);
    await expect(
      StatefulTestContract.deployWithOpts({ wallet, method: 'wrong_constructor' }).send().deployed(),
    ).rejects.toThrow(/Unknown function/);
  });
});
