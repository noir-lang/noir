import { createAccounts, getDeployedTestAccountsWallets } from '@aztec/accounts/testing';
import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import {
  AccountWalletWithPrivateKey,
  AztecNode,
  CheatCodes,
  CompleteAddress,
  DebugLogger,
  DeployL1Contracts,
  EthCheatCodes,
  L1ContractArtifactsForDeployment,
  L2BlockL2Logs,
  LogType,
  PXE,
  SentTx,
  createAztecNodeClient,
  createDebugLogger,
  createPXEClient,
  deployL1Contracts,
  waitForPXE,
} from '@aztec/aztec.js';
import {
  AvailabilityOracleAbi,
  AvailabilityOracleBytecode,
  ContractDeploymentEmitterAbi,
  ContractDeploymentEmitterBytecode,
  InboxAbi,
  InboxBytecode,
  OutboxAbi,
  OutboxBytecode,
  RegistryAbi,
  RegistryBytecode,
  RollupAbi,
  RollupBytecode,
} from '@aztec/l1-artifacts';
import { PXEService, PXEServiceConfig, createPXEService, getPXEServiceConfig } from '@aztec/pxe';
import { SequencerClient } from '@aztec/sequencer-client';

import * as path from 'path';
import {
  Account,
  Chain,
  HDAccount,
  HttpTransport,
  PrivateKeyAccount,
  createPublicClient,
  createWalletClient,
  http,
} from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

import { MNEMONIC } from './fixtures.js';
import { isMetricsLoggingRequested, setupMetricsLogger } from './logging.js';

export { deployAndInitializeTokenAndBridgeContracts } from '../shared/cross_chain_test_harness.js';

const { PXE_URL = '' } = process.env;

const getAztecUrl = () => {
  return PXE_URL;
};

export const setupL1Contracts = async (
  l1RpcUrl: string,
  account: HDAccount | PrivateKeyAccount,
  logger: DebugLogger,
) => {
  const l1Artifacts: L1ContractArtifactsForDeployment = {
    contractDeploymentEmitter: {
      contractAbi: ContractDeploymentEmitterAbi,
      contractBytecode: ContractDeploymentEmitterBytecode,
    },
    registry: {
      contractAbi: RegistryAbi,
      contractBytecode: RegistryBytecode,
    },
    inbox: {
      contractAbi: InboxAbi,
      contractBytecode: InboxBytecode,
    },
    outbox: {
      contractAbi: OutboxAbi,
      contractBytecode: OutboxBytecode,
    },
    availabilityOracle: {
      contractAbi: AvailabilityOracleAbi,
      contractBytecode: AvailabilityOracleBytecode,
    },
    rollup: {
      contractAbi: RollupAbi,
      contractBytecode: RollupBytecode,
    },
  };
  return await deployL1Contracts(l1RpcUrl, account, foundry, logger, l1Artifacts);
};

/**
 * Sets up Private eXecution Environment (PXE).
 * @param numberOfAccounts - The number of new accounts to be created once the PXE is initiated.
 * @param aztecNode - An instance of Aztec Node.
 * @param opts - Partial configuration for the PXE service.
 * @param firstPrivKey - The private key of the first account to be created.
 * @param logger - The logger to be used.
 * @param useLogSuffix - Whether to add a randomly generated suffix to the PXE debug logs.
 * @returns Private eXecution Environment (PXE), accounts, wallets and logger.
 */
export async function setupPXEService(
  numberOfAccounts: number,
  aztecNode: AztecNode,
  opts: Partial<PXEServiceConfig> = {},
  logger = getLogger(),
  useLogSuffix = false,
): Promise<{
  /**
   * The PXE instance.
   */
  pxe: PXE;
  /**
   * The accounts created by the PXE.
   */
  accounts: CompleteAddress[];
  /**
   * The wallets to be used.
   */
  wallets: AccountWalletWithPrivateKey[];
  /**
   * Logger instance named as the current test.
   */
  logger: DebugLogger;
}> {
  const pxeServiceConfig = { ...getPXEServiceConfig(), ...opts };
  const pxe = await createPXEService(aztecNode, pxeServiceConfig, useLogSuffix);

  const wallets = await createAccounts(pxe, numberOfAccounts);

  return {
    pxe,
    accounts: await pxe.getRegisteredAccounts(),
    wallets,
    logger,
  };
}

/**
 * Function to setup the test against a remote deployment. It is assumed that L1 contract are already deployed
 * @param account - The account for use in create viem wallets.
 * @param config - The aztec Node Configuration
 * @param logger - The logger to be used
 * @param numberOfAccounts - The number of new accounts to be created once the PXE is initiated.
 * (will create extra accounts if the environment doesn't already have enough accounts)
 * @returns Private eXecution Environment (PXE) client, viem wallets, contract addresses etc.
 */
async function setupWithRemoteEnvironment(
  account: Account,
  config: AztecNodeConfig,
  logger: DebugLogger,
  numberOfAccounts: number,
) {
  // we are setting up against a remote environment, l1 contracts are already deployed
  const aztecNodeUrl = getAztecUrl();
  logger(`Creating Aztec Node client to remote host ${aztecNodeUrl}`);
  const aztecNode = createAztecNodeClient(aztecNodeUrl);
  logger(`Creating PXE client to remote host ${PXE_URL}`);
  const pxeClient = createPXEClient(PXE_URL);
  await waitForPXE(pxeClient, logger);
  logger('JSON RPC client connected to PXE');
  logger(`Retrieving contract addresses from ${PXE_URL}`);
  const l1Contracts = (await pxeClient.getNodeInfo()).l1ContractAddresses;
  logger('PXE created, constructing available wallets from already registered accounts...');
  const wallets = await getDeployedTestAccountsWallets(pxeClient);

  if (wallets.length < numberOfAccounts) {
    const numNewAccounts = numberOfAccounts - wallets.length;
    logger(`Deploying ${numNewAccounts} accounts...`);
    wallets.push(...(await createAccounts(pxeClient, numNewAccounts)));
  }

  const walletClient = createWalletClient<HttpTransport, Chain, HDAccount>({
    account,
    chain: foundry,
    transport: http(config.rpcUrl),
  });
  const publicClient = createPublicClient({
    chain: foundry,
    transport: http(config.rpcUrl),
  });
  const deployL1ContractsValues: DeployL1Contracts = {
    l1ContractAddresses: l1Contracts,
    walletClient,
    publicClient,
  };
  const cheatCodes = CheatCodes.create(config.rpcUrl, pxeClient!);
  const teardown = () => Promise.resolve();
  return {
    aztecNode,
    sequencer: undefined,
    pxe: pxeClient,
    deployL1ContractsValues,
    accounts: await pxeClient!.getRegisteredAccounts(),
    config,
    wallet: wallets[0],
    wallets,
    logger,
    cheatCodes,
    teardown,
  };
}

/** Options for the e2e tests setup */
type SetupOptions = {
  /** State load */
  stateLoad?: string;
  /** Previously deployed contracts on L1 */
  deployL1ContractsValues?: DeployL1Contracts;
} & Partial<AztecNodeConfig>;

/** Context for an end-to-end test as returned by the `setup` function */
export type EndToEndContext = {
  /** The Aztec Node service or client a connected to it. */
  aztecNode: AztecNode;
  /** A client to the sequencer service (undefined if connected to remote environment) */
  sequencer: SequencerClient | undefined;
  /** The Private eXecution Environment (PXE). */
  pxe: PXE;
  /** Return values from deployL1Contracts function. */
  deployL1ContractsValues: DeployL1Contracts;
  /** The accounts created by the PXE. */
  accounts: CompleteAddress[];
  /** The Aztec Node configuration. */
  config: AztecNodeConfig;
  /** The first wallet to be used. */
  wallet: AccountWalletWithPrivateKey;
  /** The wallets to be used. */
  wallets: AccountWalletWithPrivateKey[];
  /** Logger instance named as the current test. */
  logger: DebugLogger;
  /** The cheat codes. */
  cheatCodes: CheatCodes;
  /** Function to stop the started services. */
  teardown: () => Promise<void>;
};

/**
 * Sets up the environment for the end-to-end tests.
 * @param numberOfAccounts - The number of new accounts to be created once the PXE is initiated.
 * @param opts - Options to pass to the node initialization and to the setup script.
 * @param pxeOpts - Options to pass to the PXE initialization.
 */
export async function setup(
  numberOfAccounts = 1,
  opts: SetupOptions = {},
  pxeOpts: Partial<PXEServiceConfig> = {},
): Promise<EndToEndContext> {
  const config = { ...getConfigEnvVars(), ...opts };

  // Enable logging metrics to a local file named after the test suite
  if (isMetricsLoggingRequested()) {
    const filename = path.join('log', getJobName() + '.jsonl');
    setupMetricsLogger(filename);
  }

  if (opts.stateLoad) {
    const ethCheatCodes = new EthCheatCodes(config.rpcUrl);
    await ethCheatCodes.loadChainState(opts.stateLoad);
  }

  const logger = getLogger();
  const hdAccount = mnemonicToAccount(MNEMONIC);
  const privKeyRaw = hdAccount.getHdKey().privateKey;
  const publisherPrivKey = privKeyRaw === null ? null : Buffer.from(privKeyRaw);

  if (PXE_URL) {
    // we are setting up against a remote environment, l1 contracts are assumed to already be deployed
    return await setupWithRemoteEnvironment(hdAccount, config, logger, numberOfAccounts);
  }

  const deployL1ContractsValues =
    opts.deployL1ContractsValues ?? (await setupL1Contracts(config.rpcUrl, hdAccount, logger));

  config.publisherPrivateKey = `0x${publisherPrivKey!.toString('hex')}`;
  config.l1Contracts = deployL1ContractsValues.l1ContractAddresses;

  logger('Creating and synching an aztec node...');
  const aztecNode = await AztecNodeService.createAndSync(config);
  const sequencer = aztecNode.getSequencer();

  const { pxe, accounts, wallets } = await setupPXEService(numberOfAccounts, aztecNode!, pxeOpts, logger);

  const cheatCodes = CheatCodes.create(config.rpcUrl, pxe!);

  const teardown = async () => {
    if (aztecNode instanceof AztecNodeService) {
      await aztecNode?.stop();
    }
    if (pxe instanceof PXEService) {
      await pxe?.stop();
    }
  };

  return {
    aztecNode,
    pxe,
    deployL1ContractsValues,
    accounts,
    config,
    wallet: wallets[0],
    wallets,
    logger,
    cheatCodes,
    sequencer,
    teardown,
  };
}

/**
 * Sets the timestamp of the next block.
 * @param rpcUrl - rpc url of the blockchain instance to connect to
 * @param timestamp - the timestamp for the next block
 */
export async function setNextBlockTimestamp(rpcUrl: string, timestamp: number) {
  const params = `[${timestamp}]`;
  await fetch(rpcUrl, {
    body: `{"jsonrpc":"2.0", "method": "evm_setNextBlockTimestamp", "params": ${params}, "id": 1}`,
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  });
}

/** Returns the job name for the current test. */
function getJobName() {
  return process.env.JOB_NAME ?? expect.getState().currentTestName?.split(' ')[0].replaceAll('/', '_') ?? 'unknown';
}

/**
 * Returns a logger instance for the current test.
 * @returns a logger instance for the current test.
 */
export function getLogger() {
  const describeBlockName = expect.getState().currentTestName?.split(' ')[0].replaceAll('/', ':');
  if (!describeBlockName) {
    const name = expect.getState().testPath?.split('/').pop()?.split('.')[0] ?? 'unknown';
    return createDebugLogger('aztec:' + name);
  }
  return createDebugLogger('aztec:' + describeBlockName);
}

// docs:start:delay
/**
 * Sleep for a given number of milliseconds.
 * @param ms - the number of milliseconds to sleep for
 */
export function delay(ms: number): Promise<void> {
  return new Promise<void>(resolve => setTimeout(resolve, ms));
}
// docs:end:delay

/**
 * Checks the number of encrypted logs in the last block is as expected.
 * @param aztecNode - The instance of aztec node for retrieving the logs.
 * @param numEncryptedLogs - The number of expected logs.
 */
export const expectsNumOfEncryptedLogsInTheLastBlockToBe = async (
  aztecNode: AztecNode | undefined,
  numEncryptedLogs: number,
) => {
  if (!aztecNode) {
    // An api for retrieving encrypted logs does not exist on the PXE Service so we have to use the node
    // This means we can't perform this check if there is no node
    return;
  }
  const l2BlockNum = await aztecNode.getBlockNumber();
  const encryptedLogs = await aztecNode.getLogs(l2BlockNum, 1, LogType.ENCRYPTED);
  const unrolledLogs = L2BlockL2Logs.unrollLogs(encryptedLogs);
  expect(unrolledLogs.length).toBe(numEncryptedLogs);
};

/**
 * Checks that the last block contains the given expected unencrypted log messages.
 * @param tx - An instance of SentTx for which to retrieve the logs.
 * @param logMessages - The set of expected log messages.
 */
export const expectUnencryptedLogsInTxToBe = async (tx: SentTx, logMessages: string[]) => {
  const unencryptedLogs = (await tx.getUnencryptedLogs()).logs;
  const asciiLogs = unencryptedLogs.map(extendedLog => extendedLog.log.data.toString('ascii'));

  expect(asciiLogs).toStrictEqual(logMessages);
};

/**
 * Checks that the last block contains the given expected unencrypted log messages.
 * @param pxe - An instance of PXE for retrieving the logs.
 * @param logMessages - The set of expected log messages.
 */
export const expectUnencryptedLogsFromLastBlockToBe = async (pxe: PXE, logMessages: string[]) => {
  // docs:start:get_logs
  // Get the unencrypted logs from the last block
  const fromBlock = await pxe.getBlockNumber();
  const logFilter = {
    fromBlock,
    toBlock: fromBlock + 1,
  };
  const unencryptedLogs = (await pxe.getUnencryptedLogs(logFilter)).logs;
  // docs:end:get_logs
  const asciiLogs = unencryptedLogs.map(extendedLog => extendedLog.log.data.toString('ascii'));

  expect(asciiLogs).toStrictEqual(logMessages);
};
