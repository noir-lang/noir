import { type AccountWallet, type AztecAddress, type DebugLogger, Fr } from '@aztec/aztec.js';
import { TestContract } from '@aztec/noir-contracts.js';
import { EasyPrivateVotingContract } from '@aztec/noir-contracts.js/EasyPrivateVoting';

import { setup } from './fixtures/utils.js';

const SHARED_MUTABLE_DELAY = 5;

describe('e2e_voting_contract', () => {
  let wallet: AccountWallet;

  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let testContract: TestContract;
  let votingContract: EasyPrivateVotingContract;
  let owner: AztecAddress;

  beforeAll(async () => {
    // Setup environment
    ({ teardown, wallet, logger } = await setup(1));
    owner = wallet.getAddress();

    testContract = await TestContract.deploy(wallet).send().deployed();
    votingContract = await EasyPrivateVotingContract.deploy(wallet, owner).send().deployed();

    logger.info(`Counter contract deployed at ${votingContract.address}`);
  });

  afterAll(() => teardown());

  const crossDelay = async () => {
    for (let i = 0; i < SHARED_MUTABLE_DELAY; i++) {
      // We send arbitrary tx to mine a block
      await testContract.methods.emit_unencrypted(0).send().wait();
    }
  };

  describe('votes', () => {
    it('votes, rotates nullifier keys, then tries to vote again', async () => {
      const candidate = new Fr(1);
      await votingContract.methods.cast_vote(candidate).send().wait();
      expect(await votingContract.methods.get_vote(candidate).simulate()).toBe(1n);

      // We rotate our nullifier keys - this should be ignored by the voting contract, since it should always use the
      // same set of keys to prevent double spends.
      await wallet.rotateNullifierKeys();
      await crossDelay();

      // We try voting again, but our TX is dropped due to trying to emit duplicate nullifiers as the voting contract
      // ignored our previous key rotation.
      await expect(votingContract.methods.cast_vote(candidate).send().wait()).rejects.toThrow(
        'Reason: Tx dropped by P2P node.',
      );
    });
  });
});
