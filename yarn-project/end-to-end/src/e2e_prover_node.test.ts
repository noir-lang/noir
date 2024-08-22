import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type Archiver, retrieveL2ProofVerifiedEvents } from '@aztec/archiver';
import {
  type AccountWalletWithSecretKey,
  type AztecAddress,
  type DebugLogger,
  EthAddress,
  Fr,
  SignerlessWallet,
  computeSecretHash,
  createDebugLogger,
  sleep,
} from '@aztec/aztec.js';
import { StatefulTestContract, TestContract } from '@aztec/noir-contracts.js';
import { type ProverNodeConfig, createProverNode } from '@aztec/prover-node';

import { sendL1ToL2Message } from './fixtures/l1_to_l2_messaging.js';
import {
  type ISnapshotManager,
  type SubsystemsContext,
  addAccounts,
  createSnapshotManager,
} from './fixtures/snapshot_manager.js';
import { waitForProvenChain } from './fixtures/utils.js';

// Tests simple block building with a sequencer that does not upload proofs to L1,
// and then follows with a prover node run (with real proofs disabled, but
// still simulating all circuits via a prover-client), in order to test
// the coordination through L1 between the sequencer and the prover node.
describe('e2e_prover_node', () => {
  let ctx: SubsystemsContext;
  let wallet: AccountWalletWithSecretKey;
  let recipient: AztecAddress;
  let contract: StatefulTestContract;
  let msgTestContract: TestContract;

  let logger: DebugLogger;
  let snapshotManager: ISnapshotManager;

  const msgContent: Fr = Fr.fromString('0xcafe');
  const msgSecret: Fr = Fr.fromString('0xfeca');

  beforeAll(async () => {
    logger = createDebugLogger('aztec:e2e_prover_node');
    snapshotManager = createSnapshotManager(`e2e_prover_node`, process.env.E2E_DATA_PATH);

    const testContractOpts = { contractAddressSalt: Fr.ONE, universalDeploy: true };
    await snapshotManager.snapshot(
      'send-l1-to-l2-msg',
      async ctx => {
        const testContract = TestContract.deploy(new SignerlessWallet(ctx.pxe)).getInstance(testContractOpts);
        const msgHash = await sendL1ToL2Message(
          { recipient: testContract.address, content: msgContent, secretHash: computeSecretHash(msgSecret) },
          ctx.deployL1ContractsValues,
        );
        return { msgHash };
      },
      async (_data, ctx) => {
        msgTestContract = await TestContract.deploy(new SignerlessWallet(ctx.pxe)).register(testContractOpts);
      },
    );

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

    ctx = await snapshotManager.setup();
  });

  it('submits three blocks, then prover proves the first two', async () => {
    // wait for the proven chain to catch up with the pending chain before we shut off the prover node
    await waitForProvenChain(ctx.aztecNode);

    // Stop the current prover node
    await ctx.proverNode.stop();

    const msgSender = ctx.deployL1ContractsValues.walletClient.account.address;
    const txReceipt1 = await msgTestContract.methods
      .consume_message_from_arbitrary_sender_private(msgContent, msgSecret, EthAddress.fromString(msgSender))
      .send()
      .wait();
    const txReceipt2 = await contract.methods.create_note(recipient, recipient, 10).send().wait();
    const txReceipt3 = await contract.methods.increment_public_value(recipient, 20).send().wait();

    // Check everything went well during setup and txs were mined in two different blocks
    const firstBlock = txReceipt1.blockNumber!;
    const secondBlock = firstBlock + 1;
    expect(txReceipt2.blockNumber).toEqual(secondBlock);
    expect(txReceipt3.blockNumber).toEqual(firstBlock + 2);
    expect(await contract.methods.get_public_value(recipient).simulate()).toEqual(20n);
    expect(await contract.methods.summed_values(recipient).simulate()).toEqual(10n);

    // Kick off a prover node
    await sleep(1000);
    const proverId = Fr.fromString(Buffer.from('awesome-prover', 'utf-8').toString('hex'));
    logger.info(`Creating prover node ${proverId.toString()}`);
    // HACK: We have to use the existing archiver to fetch L2 data, since anvil's chain dump/load used by the
    // snapshot manager does not include events nor txs, so a new archiver would not "see" old blocks.
    const proverConfig: ProverNodeConfig = {
      ...ctx.aztecNodeConfig,
      txProviderNodeUrl: undefined,
      dataDirectory: undefined,
      proverId,
      proverNodeMaxPendingJobs: 100,
    };
    const archiver = ctx.aztecNode.getBlockSource() as Archiver;
    const proverNode = await createProverNode(proverConfig, { aztecNodeTxProvider: ctx.aztecNode, archiver });

    // Prove the first two blocks simultaneously
    logger.info(`Starting proof for first block #${firstBlock}`);
    await proverNode.startProof(firstBlock, firstBlock);
    logger.info(`Starting proof for second block #${secondBlock}`);
    await proverNode.startProof(secondBlock, secondBlock);

    // Confirm that we cannot go back to prove an old one
    await expect(proverNode.startProof(firstBlock, firstBlock)).rejects.toThrow(/behind the current world state/i);

    // Await until proofs get submitted
    await waitForProvenChain(ctx.aztecNode, secondBlock);
    expect(await ctx.aztecNode.getProvenBlockNumber()).toEqual(secondBlock);

    // Check that the prover id made it to the emitted event
    const { publicClient, l1ContractAddresses } = ctx.deployL1ContractsValues;
    const logs = await retrieveL2ProofVerifiedEvents(publicClient, l1ContractAddresses.rollupAddress, 1n);
    expect(logs.length).toEqual(secondBlock);

    const expectedBlockNumbers = [firstBlock, secondBlock];
    const logsSlice = logs.slice(firstBlock - 1);
    for (let i = 0; i < 2; i++) {
      const log = logsSlice[i];
      expect(log.l2BlockNumber).toEqual(BigInt(expectedBlockNumbers[i]));
      expect(log.proverId.toString()).toEqual(proverId.toString());
    }
  });
});
