import { type AccountWallet, type AztecAddress, type DebugLogger } from '@aztec/aztec.js';
import { CounterContract } from '@aztec/noir-contracts.js/Counter';

import { setup } from './fixtures/utils.js';

describe('e2e_counter_contract', () => {
  let wallet: AccountWallet;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let counterContract: CounterContract;
  let owner: AztecAddress;
  let outgoingViewer: AztecAddress;

  beforeAll(async () => {
    // Setup environment
    ({ teardown, wallet, logger } = await setup(1));
    owner = wallet.getAddress();
    // Setting the outgoing viewer to owner to not have to bother with setting up another account.
    outgoingViewer = owner;

    counterContract = await CounterContract.deploy(wallet, 0, owner, outgoingViewer).send().deployed();

    logger.info(`Counter contract deployed at ${counterContract.address}`);
  });

  afterAll(() => teardown());

  describe('increments', () => {
    it('counts', async () => {
      const receipt = await counterContract.methods.increment(owner, outgoingViewer).send().wait();
      expect(await counterContract.methods.get_counter(owner).simulate()).toBe(1n);
      expect(receipt.transactionFee).toBeGreaterThan(0n);
    });
  });
});
