import { type AccountWallet, type AztecAddress, type CompleteAddress, type DebugLogger, Fr } from '@aztec/aztec.js';
import { EasyPrivateVotingContract } from '@aztec/noir-contracts.js/EasyPrivateVoting';

import { setup } from './fixtures/utils.js';

describe('e2e_voting_contract', () => {
  let wallet: AccountWallet;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let votingContract: EasyPrivateVotingContract;
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

    votingContract = await EasyPrivateVotingContract.deploy(wallet, owner).send().deployed();

    logger(`Counter contract deployed at ${votingContract.address}`);
  }, 25_000);

  afterAll(() => teardown());

  describe('votes', () => {
    it('votes', async () => {
      const candidate = new Fr(1);
      await votingContract.methods.cast_vote(candidate).send().wait();
      expect(await votingContract.methods.get_vote(candidate).simulate()).toBe(1n);
    });
  });
});
