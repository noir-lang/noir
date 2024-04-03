import { SchnorrAccountContractArtifact } from '@aztec/accounts/schnorr';
import { createAccounts, getDeployedTestAccountsWallets } from '@aztec/accounts/testing';
import { type AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import {
  type AccountWalletWithPrivateKey,
  type AztecAddress,
  type AztecNode,
  BatchCall,
  CheatCodes,
  type CompleteAddress,
  type ContractMethod,
  type DebugLogger,
  type DeployL1Contracts,
  EncryptedL2BlockL2Logs,
  EthCheatCodes,
  type L1ContractArtifactsForDeployment,
  LogType,
  type PXE,
  type SentTx,
  SignerlessWallet,
  type Wallet,
  createAztecNodeClient,
  createDebugLogger,
  createPXEClient,
  deployL1Contracts,
  fileURLToPath,
  makeFetch,
  waitForPXE,
} from '@aztec/aztec.js';
import { deployInstance, registerContractClass } from '@aztec/aztec.js/deployment';
import { DefaultMultiCallEntrypoint } from '@aztec/entrypoints/multi-call';
import { randomBytes } from '@aztec/foundation/crypto';
import {
  AvailabilityOracleAbi,
  AvailabilityOracleBytecode,
  GasPortalAbi,
  GasPortalBytecode,
  InboxAbi,
  InboxBytecode,
  OutboxAbi,
  OutboxBytecode,
  PortalERC20Abi,
  PortalERC20Bytecode,
  RegistryAbi,
  RegistryBytecode,
  RollupAbi,
  RollupBytecode,
} from '@aztec/l1-artifacts';
import { getCanonicalGasToken, getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';
import { PXEService, type PXEServiceConfig, createPXEService, getPXEServiceConfig } from '@aztec/pxe';
import { type SequencerClient } from '@aztec/sequencer-client';

import * as fs from 'fs/promises';
import * as path from 'path';
import {
  type Account,
  type Chain,
  type HDAccount,
  type HttpTransport,
  type PrivateKeyAccount,
  createPublicClient,
  createWalletClient,
  getContract,
  http,
} from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

import { MNEMONIC } from './fixtures.js';
import { isMetricsLoggingRequested, setupMetricsLogger } from './logging.js';

export { deployAndInitializeTokenAndBridgeContracts } from '../shared/cross_chain_test_harness.js';

const {
  PXE_URL = '',
  NOIR_RELEASE_DIR = 'noir-repo/target/release',
  TEMP_DIR = '/tmp',
  ACVM_BINARY_PATH = '',
  ACVM_WORKING_DIRECTORY = '',
  ENABLE_GAS = '',
} = process.env;

const getAztecUrl = () => {
  return PXE_URL;
};

// Determines if we have access to the acvm binary and a tmp folder for temp files
const getACVMConfig = async (logger: DebugLogger) => {
  try {
    const expectedAcvmPath = ACVM_BINARY_PATH
      ? ACVM_BINARY_PATH
      : `${path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../noir/', NOIR_RELEASE_DIR)}/acvm`;
    await fs.access(expectedAcvmPath, fs.constants.R_OK);
    const tempWorkingDirectory = `${TEMP_DIR}/${randomBytes(4).toString('hex')}`;
    const acvmWorkingDirectory = ACVM_WORKING_DIRECTORY ? ACVM_WORKING_DIRECTORY : `${tempWorkingDirectory}/acvm`;
    await fs.mkdir(acvmWorkingDirectory, { recursive: true });
    logger(`Using native ACVM binary at ${expectedAcvmPath} with working directory ${acvmWorkingDirectory}`);
    return {
      acvmWorkingDirectory,
      expectedAcvmPath,
      directoryToCleanup: ACVM_WORKING_DIRECTORY ? undefined : tempWorkingDirectory,
    };
  } catch (err) {
    logger(`Native ACVM not available, error: ${err}`);
    return undefined;
  }
};

export const setupL1Contracts = async (
  l1RpcUrl: string,
  account: HDAccount | PrivateKeyAccount,
  logger: DebugLogger,
) => {
  const l1Artifacts: L1ContractArtifactsForDeployment = {
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
    gasToken: {
      contractAbi: PortalERC20Abi,
      contractBytecode: PortalERC20Bytecode,
    },
    gasPortal: {
      contractAbi: GasPortalAbi,
      contractBytecode: GasPortalBytecode,
    },
  };

  const l1Data = await deployL1Contracts(l1RpcUrl, account, foundry, logger, l1Artifacts);
  await initGasBridge(l1Data);

  return l1Data;
};

async function initGasBridge({ walletClient, l1ContractAddresses }: DeployL1Contracts) {
  const gasPortal = getContract({
    address: l1ContractAddresses.gasPortalAddress.toString(),
    abi: GasPortalAbi,
    client: walletClient,
  });

  await gasPortal.write.initialize(
    [
      l1ContractAddresses.registryAddress.toString(),
      l1ContractAddresses.gasTokenAddress.toString(),
      getCanonicalGasTokenAddress(l1ContractAddresses.gasPortalAddress).toString(),
    ],
    {} as any,
  );
}

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
  const pxeClient = createPXEClient(PXE_URL, makeFetch([1, 2, 3], true));
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

  if (['1', 'true'].includes(ENABLE_GAS)) {
    // this contract might already have been deployed
    // the following function is idempotent
    await deployCanonicalGasToken(new SignerlessWallet(pxeClient, new DefaultMultiCallEntrypoint()));
  }

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

  const acvmConfig = await getACVMConfig(logger);
  if (acvmConfig) {
    config.acvmWorkingDirectory = acvmConfig.acvmWorkingDirectory;
    config.acvmBinaryPath = acvmConfig.expectedAcvmPath;
  }
  config.l1BlockPublishRetryIntervalMS = 100;
  const aztecNode = await AztecNodeService.createAndSync(config);
  const sequencer = aztecNode.getSequencer();

  const { pxe, accounts, wallets } = await setupPXEService(numberOfAccounts, aztecNode!, pxeOpts, logger);

  if (['1', 'true'].includes(ENABLE_GAS)) {
    await deployCanonicalGasToken(new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint()));
  }

  const cheatCodes = CheatCodes.create(config.rpcUrl, pxe!);

  const teardown = async () => {
    if (aztecNode instanceof AztecNodeService) {
      await aztecNode?.stop();
    }
    if (pxe instanceof PXEService) {
      await pxe?.stop();
    }

    if (acvmConfig?.directoryToCleanup) {
      // remove the temp directory created for the acvm
      logger(`Cleaning up ACVM temp directory ${acvmConfig.directoryToCleanup}`);
      await fs.rm(acvmConfig.directoryToCleanup, { recursive: true, force: true });
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
 * Registers the contract class used for test accounts and publicly deploys the instances requested.
 * Use this when you need to make a public call to an account contract, such as for requesting a public authwit.
 * @param sender - Wallet to send the deployment tx.
 * @param accountsToDeploy - Which accounts to publicly deploy.
 */
export async function publicDeployAccounts(sender: Wallet, accountsToDeploy: (CompleteAddress | AztecAddress)[]) {
  const accountAddressesToDeploy = accountsToDeploy.map(a => ('address' in a ? a.address : a));
  const instances = await Promise.all(accountAddressesToDeploy.map(account => sender.getContractInstance(account)));
  const batch = new BatchCall(sender, [
    (await registerContractClass(sender, SchnorrAccountContractArtifact)).request(),
    ...instances.map(instance => deployInstance(sender, instance!).request()),
  ]);
  await batch.send().wait();
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
  const unrolledLogs = EncryptedL2BlockL2Logs.unrollLogs(encryptedLogs);
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

export type BalancesFn = ReturnType<typeof getBalancesFn>;
export function getBalancesFn(
  symbol: string,
  method: ContractMethod,
  logger: any,
): (...addresses: AztecAddress[]) => Promise<bigint[]> {
  const balances = async (...addresses: AztecAddress[]) => {
    const b = await Promise.all(addresses.map(address => method(address).simulate()));
    const debugString = `${symbol} balances: ${addresses.map((address, i) => `${address}: ${b[i]}`).join(', ')}`;
    logger(debugString);
    return b;
  };

  return balances;
}

export async function expectMapping<K, V>(
  fn: (...k: K[]) => Promise<V[]>,
  inputs: K[],
  expectedOutputs: V[],
): Promise<void> {
  expect(inputs.length).toBe(expectedOutputs.length);

  const outputs = await fn(...inputs);

  expect(outputs).toEqual(expectedOutputs);
}

/**
 * Deploy the protocol contracts to a running instance.
 */
export async function deployCanonicalGasToken(deployer: Wallet) {
  // "deploy" the Gas token as it contains public functions
  const gasPortalAddress = (await deployer.getNodeInfo()).l1ContractAddresses.gasPortalAddress;
  const canonicalGasToken = getCanonicalGasToken(gasPortalAddress);

  if (await deployer.isContractClassPubliclyRegistered(canonicalGasToken.contractClass.id)) {
    return;
  }

  await new BatchCall(deployer, [
    (await registerContractClass(deployer, canonicalGasToken.artifact)).request(),
    deployInstance(deployer, canonicalGasToken.instance).request(),
  ])
    .send()
    .wait();

  await expect(deployer.isContractClassPubliclyRegistered(canonicalGasToken.contractClass.id)).resolves.toBe(true);
  await expect(deployer.getContractInstance(canonicalGasToken.instance.address)).resolves.toBeDefined();
}
