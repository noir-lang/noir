import { DebugLogger, Fr, Wallet } from '@aztec/aztec.js';
import { AvmTestContract } from '@aztec/noir-contracts.js';

import { setup } from './fixtures/utils.js';

process.env.AVM_ENABLED = 'absofrigginlutely';

describe('e2e_nested_contract', () => {
  let wallet: Wallet;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  beforeEach(async () => {
    ({ teardown, wallet, logger } = await setup());
  }, 100_000);

  afterEach(() => teardown());

  describe('Call succeeds through AVM', () => {
    let avmContact: AvmTestContract;

    beforeEach(async () => {
      avmContact = await AvmTestContract.deploy(wallet).send().deployed();
    }, 50_000);

    it('Calls an avm contract', async () => {
      const a = new Fr(1);
      const b = new Fr(2);

      logger('Calling avm_addArgsReturn...');
      await avmContact.methods.avm_addArgsReturn(a, b).send().wait();
      logger('Success');
    });
  });
});
