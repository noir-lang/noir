import { type AccountWallet, type AztecAddress, type DebugLogger } from '@aztec/aztec.js';
import { CounterContract } from '@aztec/noir-contracts.js/Counter';

import { setup } from './fixtures/utils.js';

describe('e2e_counter_contract', () => {
  let wallet: AccountWallet;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let counterContract: CounterContract;
  let owner: AztecAddress;

  beforeAll(async () => {
    // Setup environment
    ({ teardown, wallet, logger } = await setup(1));
    owner = wallet.getAddress();

    counterContract = await CounterContract.deploy(wallet, 0, owner).send().deployed();

    logger.info(`Counter contract deployed at ${counterContract.address}`);
  }, 45_000);

  afterAll(() => teardown());

  describe('increments', () => {
    it('counts', async () => {
      await counterContract.methods.increment(owner).send().wait();
      expect(await counterContract.methods.get_counter(owner).simulate()).toBe(1n);
    });
  });
});
