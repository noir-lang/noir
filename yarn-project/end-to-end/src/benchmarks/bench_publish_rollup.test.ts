/* eslint-disable camelcase */
import { AztecAddress } from '@aztec/aztec.js';
import { sleep } from '@aztec/foundation/sleep';
import { TokenContract } from '@aztec/noir-contracts/types';

import times from 'lodash.times';

import { setup } from '../fixtures/utils.js';

const ROLLUP_SIZES = process.env.ROLLUP_SIZES ? process.env.ROLLUP_SIZES.split(',').map(Number) : [8, 32, 128];

describe('benchmarks/publish_rollup', () => {
  let context: Awaited<ReturnType<typeof setup>>;
  let token: TokenContract;
  let owner: AztecAddress;
  let recipient: AztecAddress;

  beforeEach(async () => {
    context = await setup(2, { maxTxsPerBlock: 1024 });
    [owner, recipient] = context.accounts.map(a => a.address);
    token = await TokenContract.deploy(context.wallet, owner).send().deployed();
    await token.methods.mint_public(owner, 10000n).send().wait();
    await context.aztecNode?.getSequencer().stop();
  }, 60_000);

  it.each(ROLLUP_SIZES)(
    `publishes a rollup with %d txs`,
    async (txCount: number) => {
      context.logger(`Assembling rollup with ${txCount} txs`);
      // Simulate and simultaneously send %d txs. These should not yet be processed since sequencer is stopped.
      const calls = times(txCount, () => token.methods.transfer_public(owner, recipient, 1, 0));
      calls.forEach(call => call.simulate({ skipPublicSimulation: true }));
      const sentTxs = calls.map(call => call.send());
      // Awaiting txHash waits until the aztec node has received the tx into its p2p pool
      await Promise.all(sentTxs.map(tx => tx.getTxHash()));
      // And then wait a bit more just in case
      await sleep(100);
      // Restart sequencer to process all txs together
      context.aztecNode?.getSequencer().restart();
      // Wait for the last tx to be processed
      await sentTxs[sentTxs.length - 1].wait({ timeout: 600_00 });
    },
    10 * 60_000,
  );

  afterEach(async () => {
    await context.teardown();
  }, 60_000);
});
