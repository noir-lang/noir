import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type Archiver } from '@aztec/archiver';
import {
  type AccountWalletWithSecretKey,
  type AztecAddress,
  type DebugLogger,
  type FieldsOf,
  type TxReceipt,
  createDebugLogger,
  retryUntil,
  sleep,
} from '@aztec/aztec.js';
import { StatefulTestContract } from '@aztec/noir-contracts.js';
import { createProverNode } from '@aztec/prover-node';
import { type SequencerClientConfig } from '@aztec/sequencer-client';

import {
  type ISnapshotManager,
  type SubsystemsContext,
  addAccounts,
  createSnapshotManager,
} from './fixtures/snapshot_manager.js';

// Tests simple block building with a sequencer that does not upload proofs to L1,
// and then follows with a prover node run (with real proofs disabled, but
// still simulating all circuits via a prover-client), in order to test
// the coordination between the sequencer and the prover node.
describe('e2e_prover_node', () => {
  let ctx: SubsystemsContext;
  let wallet: AccountWalletWithSecretKey;
  let recipient: AztecAddress;
  let contract: StatefulTestContract;
  let txReceipts: FieldsOf<TxReceipt>[];

  let logger: DebugLogger;

  let snapshotManager: ISnapshotManager;

  beforeAll(async () => {
    logger = createDebugLogger('aztec:e2e_prover_node');
    const config: Partial<SequencerClientConfig> = { sequencerSkipSubmitProofs: true };
    snapshotManager = createSnapshotManager(`e2e_prover_node`, process.env.E2E_DATA_PATH, config);

    await snapshotManager.snapshot('setup', addAccounts(2, logger), async ({ accountKeys }, ctx) => {
      const accountManagers = accountKeys.map(ak => getSchnorrAccount(ctx.pxe, ak[0], ak[1], 1));
      await Promise.all(accountManagers.map(a => a.register()));
      const wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
      wallets.forEach((w, i) => logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
      wallet = wallets[0];
      recipient = wallets[1].getAddress();
    });

    await snapshotManager.snapshot(
      'deploy-test-contract',
      async () => {
        const owner = wallet.getAddress();
        const contract = await StatefulTestContract.deploy(wallet, owner, owner, 42).send().deployed();
        return { contractAddress: contract.address };
      },
      async ({ contractAddress }) => {
        contract = await StatefulTestContract.at(contractAddress, wallet);
      },
    );

    await snapshotManager.snapshot(
      'create-blocks',
      async () => {
        const txReceipt1 = await contract.methods.create_note(recipient, recipient, 10).send().wait();
        const txReceipt2 = await contract.methods.increment_public_value(recipient, 20).send().wait();
        return { txReceipt1, txReceipt2 };
      },
      ({ txReceipt1, txReceipt2 }) => {
        txReceipts = [txReceipt1, txReceipt2];
        return Promise.resolve();
      },
    );

    ctx = await snapshotManager.setup();
  });

  it('submits two blocks, then prover proves the first one', async () => {
    // Check everything went well during setup and txs were mined in two different blocks
    const [txReceipt1, txReceipt2] = txReceipts;
    const firstBlock = txReceipt1.blockNumber!;
    expect(txReceipt2.blockNumber).toEqual(firstBlock + 1);
    expect(await contract.methods.get_public_value(recipient).simulate()).toEqual(20n);
    expect(await contract.methods.summed_values(recipient).simulate()).toEqual(10n);
    expect(await ctx.aztecNode.getProvenBlockNumber()).toEqual(0);

    // Trick archiver into thinking everything has been proven up to this point.
    // TODO: Add cheat code to flag current block as proven on L1, which will be needed when we assert on L1 that proofs do not have any gaps.
    await (ctx.aztecNode.getBlockSource() as Archiver).setProvenBlockNumber(firstBlock - 1);
    expect(await ctx.aztecNode.getProvenBlockNumber()).toEqual(firstBlock - 1);

    // Kick off a prover node
    await sleep(1000);
    logger.info('Creating prover node');
    // HACK: We have to use the existing archiver to fetch L2 data, since anvil's chain dump/load used by the
    // snapshot manager does not include events nor txs, so a new archiver would not "see" old blocks.
    const proverConfig = { ...ctx.aztecNodeConfig, txProviderNodeUrl: undefined, dataDirectory: undefined };
    const archiver = ctx.aztecNode.getBlockSource() as Archiver;
    const proverNode = await createProverNode(proverConfig, { aztecNodeTxProvider: ctx.aztecNode, archiver });

    // Prove block from first tx and block until it is proven
    logger.info(`Proving block ${firstBlock}`);
    await proverNode.prove(firstBlock, firstBlock);

    logger.info(`Proof submitted. Awaiting aztec node to sync...`);
    await retryUntil(async () => (await ctx.aztecNode.getProvenBlockNumber()) === firstBlock, 'proven-block', 10, 1);
    expect(await ctx.aztecNode.getProvenBlockNumber()).toEqual(firstBlock);
  });
});
