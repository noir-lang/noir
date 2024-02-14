import { AccountWallet, AztecAddress, CompleteAddress, DebugLogger, TxStatus } from '@aztec/aztec.js';
import { CounterContract } from '@aztec/noir-contracts.js/Counter';

import { setup } from './fixtures/utils.js';

describe('e2e_counter_contract', () => {
  let wallet: AccountWallet;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let counterContract: CounterContract;
  let owner: AztecAddress;

  beforeAll(async () => {
    // Setup environment
    ({
      teardown,
      accounts,
      wallets: [wallet],
      logger,
    } = await setup(1));
    owner = accounts[0].address;

    counterContract = await CounterContract.deploy(wallet, 0, owner).send().deployed();

    logger(`Counter contract deployed at ${counterContract.address}`);
  }, 25_000);

  afterAll(() => teardown());

  describe('increments', () => {
    it('counts', async () => {
      const receipt = await counterContract.methods.increment(owner).send().wait();
      expect(receipt.status).toBe(TxStatus.MINED);
      expect(await counterContract.methods.get_counter(owner).view()).toBe(1n);
    });
  });
});
