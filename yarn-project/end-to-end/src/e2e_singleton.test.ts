import { Fr, Wallet } from '@aztec/aztec.js';
import { DocsExampleContract } from '@aztec/noir-contracts';

import { setup } from './fixtures/utils.js';

describe('e2e_singleton', () => {
  let wallet: Wallet;

  let teardown: () => Promise<void>;
  let contract: DocsExampleContract;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
    contract = await DocsExampleContract.deploy(wallet).send().deployed();
    // sets card value to 1 and leader to sender.
    await contract.methods.initialize_private(Fr.random(), 1).send().wait();
  }, 25_000);

  afterAll(() => teardown());

  // Singleton tests:
  it('can read singleton and replace/update it in the same call', async () => {
    await expect(contract.methods.update_legendary_card(Fr.random(), 0).simulate()).rejects.toThrowError(
      'Assertion failed: can only update to higher value',
    );

    const newPoints = 3n;
    await contract.methods.update_legendary_card(Fr.random(), newPoints).send().wait();
    expect((await contract.methods.get_leader().view()).points).toEqual(newPoints);
  });
});
