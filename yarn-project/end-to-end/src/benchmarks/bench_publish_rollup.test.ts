/* eslint-disable camelcase */
import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, BatchCall } from '@aztec/aztec.js';
import { EthAddress, Fr, GrumpkinScalar } from '@aztec/circuits.js';
import { retryUntil } from '@aztec/foundation/retry';
import { sleep } from '@aztec/foundation/sleep';
import { BenchmarkingContract } from '@aztec/noir-contracts/types';
import { createPXEService } from '@aztec/pxe';
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
    context.logger(`Deployed benchmarking contract at ${contract.address}`);
    sequencer = (context.aztecNode as AztecNodeService).getSequencer()!;
    await sequencer.stop();
  }, 60_000);

  // Each tx has a private execution (account entrypoint), a nested private call (create_note),
  // a public call (increment_balance), and a nested public call (broadcast). These include
  // emitting one private note and one unencrypted log, two storage reads and one write.
  const makeBatchCall = (i: number) =>
    new BatchCall(context.wallet, [
      contract.methods.create_note(owner, i + 1).request(),
      contract.methods.increment_balance(owner, i + 1).request(),
    ]);

  it.each(ROLLUP_SIZES)(
    `publishes a rollup with %d txs`,
    async (txCount: number) => {
      context.logger(`Assembling rollup with ${txCount} txs`);
      // Simulate and simultaneously send ROLLUP_SIZE txs. These should not yet be processed since sequencer is stopped.
      const calls = times(txCount, makeBatchCall);
      calls.forEach(call => call.simulate({ skipPublicSimulation: true }));
      const sentTxs = calls.map(call => call.send());

      // Awaiting txHash waits until the aztec node has received the tx into its p2p pool
      await Promise.all(sentTxs.map(tx => tx.getTxHash()));
      await sleep(100);

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
      const pxe = await createPXEService(node, { l2BlockPollingIntervalMS: 100, l2StartingBlock: blockNumber! - 1 });
      await pxe.addContracts([{ ...contract, portalContract: EthAddress.ZERO }]);
      await retryUntil(() => pxe.isGlobalStateSynchronized(), 'pxe-global-sync');
      const { publicKey, partialAddress } = context.wallet.getCompleteAddress();
      const privateKey = context.wallet.getEncryptionPrivateKey();
      const l2Block = await node.getBlockNumber();

      // Register the owner account and wait until it's synced so we measure how much time it took
      context.logger(`Registering owner account on new pxe`);
      await pxe.registerAccount(privateKey, partialAddress);
      const isOwnerSynced = async () => (await pxe.getSyncStatus()).notes[publicKey.toString()] === l2Block;
      await retryUntil(isOwnerSynced, 'pxe-owner-sync');

      // Repeat for another account that didn't receive any notes for them, so we measure trial-decrypts
      context.logger(`Registering fresh account on new pxe`);
      const newAccount = await pxe.registerAccount(GrumpkinScalar.random(), Fr.random());
      const isNewAccountSynced = async () =>
        (await pxe.getSyncStatus()).notes[newAccount.publicKey.toString()] === l2Block;
      await retryUntil(isNewAccountSynced, 'pxe-new-account-sync');

      // Stop the external node and pxe
      await pxe.stop();
      await node.stop();
    },
    10 * 60_000,
  );
});
