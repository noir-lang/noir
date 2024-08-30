import { createAccounts } from '@aztec/accounts/testing';
import { type AztecNodeConfig } from '@aztec/aztec-node';
import { type AztecNode, type DebugLogger, Fr, type PXE } from '@aztec/aztec.js';
import { NULL_KEY } from '@aztec/ethereum';
import { EasyPrivateTokenContract } from '@aztec/noir-contracts.js';
import { type ProverNode, type ProverNodeConfig, getProverNodeConfigFromEnv } from '@aztec/prover-node';

import { foundry, sepolia } from 'viem/chains';

import { createAndSyncProverNode } from '../fixtures/snapshot_manager.js';
import { getPrivateKeyFromIndex, setup } from '../fixtures/utils.js';

// process.env.SEQ_PUBLISHER_PRIVATE_KEY = '<PRIVATE_KEY_WITH_SEPOLIA_ETH>';
// process.env.PROVER_PUBLISHER_PRIVATE_KEY = '<PRIVATE_KEY_WITH_SEPOLIA_ETH>';
// process.env.ETHEREUM_HOST= 'https://sepolia.infura.io/v3/<API_KEY>';
// process.env.L1_CHAIN_ID = '11155111';

describe(`deploys and transfers a private only token`, () => {
  let secretKey1: Fr;
  let secretKey2: Fr;
  let proverConfig: ProverNodeConfig;
  let config: AztecNodeConfig;
  let aztecNode: AztecNode;
  let proverNode: ProverNode;

  let pxe: PXE;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  beforeEach(async () => {
    const chainId = !process.env.L1_CHAIN_ID ? foundry.id : +process.env.L1_CHAIN_ID;
    const chain = chainId == sepolia.id ? sepolia : foundry; // Not the best way of doing this.
    ({ logger, pxe, teardown, config, aztecNode } = await setup(
      0,
      { skipProtocolContracts: true, stateLoad: undefined },
      {},
      false,
      chain,
    ));
    proverConfig = getProverNodeConfigFromEnv();
    const proverNodePrivateKey = getPrivateKeyFromIndex(2);
    proverConfig.publisherPrivateKey =
      proverConfig.publisherPrivateKey === NULL_KEY
        ? `0x${proverNodePrivateKey?.toString('hex')}`
        : proverConfig.publisherPrivateKey;

    proverNode = await createAndSyncProverNode(
      config.l1Contracts.rollupAddress,
      proverConfig.publisherPrivateKey,
      config,
      aztecNode,
    );
  }, 600_000);

  afterEach(async () => {
    await proverNode.stop();
    await teardown();
  });

  it('calls a private function', async () => {
    const initialBalance = 100000000000n;
    const transferValue = 5n;
    secretKey1 = Fr.random();
    secretKey2 = Fr.random();

    logger.info(`Deploying accounts.`);

    const accounts = await createAccounts(pxe, 2, [secretKey1, secretKey2], {
      interval: 0.1,
      proven: true,
      provenTimeout: 600,
    });

    logger.info(`Accounts deployed, deploying token.`);

    const [deployerWallet, recipientWallet] = accounts;

    const token = await EasyPrivateTokenContract.deploy(
      deployerWallet,
      initialBalance,
      deployerWallet.getAddress(),
      deployerWallet.getAddress(),
    )
      .send({
        universalDeploy: true,
        skipPublicDeployment: true,
        skipClassRegistration: true,
        skipInitialization: false,
        skipPublicSimulation: true,
      })
      .deployed({
        proven: true,
        provenTimeout: 600,
      });

    logger.info(`Performing transfer.`);

    await token.methods
      .transfer(transferValue, deployerWallet.getAddress(), recipientWallet.getAddress(), deployerWallet.getAddress())
      .send()
      .wait({ proven: true, provenTimeout: 600 });

    logger.info(`Transfer completed`);

    const balanceDeployer = await token.methods.get_balance(deployerWallet.getAddress()).simulate();
    const balanceRecipient = await token.methods.get_balance(recipientWallet.getAddress()).simulate();

    logger.info(`Deployer balance: ${balanceDeployer}, Recipient balance: ${balanceRecipient}`);

    expect(balanceDeployer).toBe(initialBalance - transferValue);
    expect(balanceRecipient).toBe(transferValue);
  }, 600_000);
});
