import { AztecNodeService } from '@aztec/aztec-node';
import { Fr, GrumpkinScalar } from '@aztec/circuits.js';
import { BenchmarkingContract } from '@aztec/noir-contracts/types';
import { SequencerClient } from '@aztec/sequencer-client';

import { EndToEndContext } from '../fixtures/utils.js';
import { benchmarkSetup, sendTxs, waitNewPXESynced, waitRegisteredAccountSynced } from './utils.js';

const ROLLUP_SIZES = process.env.ROLLUP_SIZES ? process.env.ROLLUP_SIZES.split(',').map(Number) : [8, 32, 128];

describe('benchmarks/publish_rollup', () => {
  let context: EndToEndContext;
  let contract: BenchmarkingContract;
  let sequencer: SequencerClient;

  beforeEach(async () => {
    ({ context, contract, sequencer } = await benchmarkSetup({ maxTxsPerBlock: 1024 }));
  }, 60_000);

  it.each(ROLLUP_SIZES)(
    `publishes a rollup with %d txs`,
    async (txCount: number) => {
      await sequencer.stop();

      // Simulate and simultaneously send ROLLUP_SIZE txs. These should not yet be processed since sequencer is stopped.
      context.logger(`Assembling rollup with ${txCount} txs`);
      const sentTxs = await sendTxs(txCount, context, contract);

      // Restart sequencer to process all txs together
      sequencer.restart();

      // Wait for the last tx to be processed and stop the current node
      const { blockNumber } = await sentTxs[sentTxs.length - 1].wait({ timeout: 5 * 60_000 });
      await context.teardown();

      // Create a new aztec node to measure sync time of the block
      // and call getTreeRoots to force a sync with world state to ensure the node has caught up
      context.logger(`Starting new aztec node`);
      const node = await AztecNodeService.createAndSync({ ...context.config, disableSequencer: true });
      await node.getTreeRoots();

      // Spin up a new pxe and sync it, we'll use it to test sync times of new accounts for the last block
      context.logger(`Starting new pxe`);
      const pxe = await waitNewPXESynced(node, contract, blockNumber! - 1);

      // Register the owner account and wait until it's synced so we measure how much time it took
      context.logger(`Registering owner account on new pxe`);
      const partialAddress = context.wallet.getCompleteAddress().partialAddress;
      const privateKey = context.wallet.getEncryptionPrivateKey();
      await waitRegisteredAccountSynced(pxe, privateKey, partialAddress);

      // Repeat for another account that didn't receive any notes for them, so we measure trial-decrypts
      context.logger(`Registering fresh account on new pxe`);
      await waitRegisteredAccountSynced(pxe, GrumpkinScalar.random(), Fr.random());

      // Stop the external node and pxe
      await pxe.stop();
      await node.stop();
    },
    10 * 60_000,
  );
});
