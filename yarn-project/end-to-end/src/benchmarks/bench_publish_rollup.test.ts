/* eslint-disable camelcase */
import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, BatchCall } from '@aztec/aztec.js';
import { sleep } from '@aztec/foundation/sleep';
import { BenchmarkingContract } from '@aztec/noir-contracts/types';
import { SequencerClient } from '@aztec/sequencer-client';

import times from 'lodash.times';

import { setup } from '../fixtures/utils.js';

const ROLLUP_SIZES = process.env.ROLLUP_SIZES ? process.env.ROLLUP_SIZES.split(',').map(Number) : [8, 32, 128];

describe('benchmarks/publish_rollup', () => {
  let context: Awaited<ReturnType<typeof setup>>;
  let contract: BenchmarkingContract;
  let owner: AztecAddress;
  let sequencer: SequencerClient;

  beforeEach(async () => {
    context = await setup(2, { maxTxsPerBlock: 1024 });
    [owner] = context.accounts.map(a => a.address);
    contract = await BenchmarkingContract.deploy(context.wallet).send().deployed();
    sequencer = (context.aztecNode as AztecNodeService).getSequencer()!;
    await sequencer.stop();
  }, 60_000);

  const makeBatchCall = (i: number) =>
    new BatchCall(context.wallet, [
      contract.methods.create_note(owner, i).request(),
      contract.methods.increment_balance(owner, i).request(),
    ]);

  it.each(ROLLUP_SIZES)(
    `publishes a rollup with %d txs`,
    async (txCount: number) => {
      context.logger(`Assembling rollup with ${txCount} txs`);
      // Simulate and simultaneously send %d txs. These should not yet be processed since sequencer is stopped.
      // Each tx has a private execution (account entrypoint), a nested private call (create_note),
      // a public call (increment_balance), and a nested public call (broadcast). These include
      // emitting one private note and one unencrypted log, two storage reads and one write.
      const calls = times(txCount, makeBatchCall);
      calls.forEach(call => call.simulate({ skipPublicSimulation: true }));
      const sentTxs = calls.map(call => call.send());

      // Awaiting txHash waits until the aztec node has received the tx into its p2p pool
      await Promise.all(sentTxs.map(tx => tx.getTxHash()));
      // And then wait a bit more just in case
      await sleep(100);

      // Restart sequencer to process all txs together
      sequencer.restart();
      // Wait for the last tx to be processed and finish the current node
      await sentTxs[sentTxs.length - 1].wait({ timeout: 600_00 });
      await context.teardown();

      // Create a new aztec node to measure sync time of the block
      context.logger(`Starting new aztec node`);
      const node = await AztecNodeService.createAndSync({ ...context.config, disableSequencer: true });
      // Force a sync with world state to ensure new node has caught up before killing it
      await node.getTreeRoots();
      await node.stop();
    },
    10 * 60_000,
  );
});
