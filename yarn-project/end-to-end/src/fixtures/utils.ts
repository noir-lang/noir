import { SchnorrAccountContractArtifact } from '@aztec/accounts/schnorr';
import { createAccounts, getDeployedTestAccountsWallets } from '@aztec/accounts/testing';
import { type AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import {
  type AccountWalletWithSecretKey,
  AztecAddress,
  type AztecNode,
  BatchCall,
  CheatCodes,
  type ContractMethod,
  type DebugLogger,
  type DeployL1Contracts,
  EncryptedNoteL2BlockL2Logs,
  EthCheatCodes,
  type L1ContractArtifactsForDeployment,
  LogType,
  NoFeePaymentMethod,
  type PXE,
  type SentTx,
  SignerlessWallet,
  type Wallet,
  createAztecNodeClient,
  createDebugLogger,
  createPXEClient,
  deployL1Contracts,
  makeFetch,
  waitForPXE,
} from '@aztec/aztec.js';
import { deployInstance, registerContractClass } from '@aztec/aztec.js/deployment';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { type BBNativePrivateKernelProver } from '@aztec/bb-prover';
import {
  CANONICAL_AUTH_REGISTRY_ADDRESS,
  CANONICAL_KEY_REGISTRY_ADDRESS,
  GasSettings,
  MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS,
  computeContractAddressFromInstance,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import { bufferAsFields } from '@aztec/foundation/abi';
import { makeBackoff, retry } from '@aztec/foundation/retry';
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
import { AuthRegistryContract, KeyRegistryContract } from '@aztec/noir-contracts.js';
import { GasTokenContract } from '@aztec/noir-contracts.js/GasToken';
import { getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';
import { getCanonicalAuthRegistry } from '@aztec/protocol-contracts/auth-registry';
import { GasTokenAddress, getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalKeyRegistry } from '@aztec/protocol-contracts/key-registry';
import { type ProverClient } from '@aztec/prover-client';
import { PXEService, type PXEServiceConfig, createPXEService, getPXEServiceConfig } from '@aztec/pxe';
import { type SequencerClient } from '@aztec/sequencer-client';
import { createAndStartTelemetryClient, getConfigEnvVars as getTelemetryConfig } from '@aztec/telemetry-client/start';

import { type Anvil, createAnvil } from '@viem/anvil';
import getPort from 'get-port';
import * as path from 'path';
import {
  type Account,
  type Chain,
  type HDAccount,
  type HttpTransport,
  type PrivateKeyAccount,
  createPublicClient,
  createWalletClient,
  http,
} from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

import { MNEMONIC } from './fixtures.js';
import { getACVMConfig } from './get_acvm_config.js';
import { getBBConfig } from './get_bb_config.js';
import { isMetricsLoggingRequested, setupMetricsLogger } from './logging.js';

export { deployAndInitializeTokenAndBridgeContracts } from '../shared/cross_chain_test_harness.js';

const { PXE_URL = '' } = process.env;

const telemetry = createAndStartTelemetryClient(getTelemetryConfig());
if (typeof afterAll === 'function') {
  afterAll(async () => {
    await telemetry.stop();
  });
}

const getAztecUrl = () => {
  return PXE_URL;
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

  const l1Data = await deployL1Contracts(l1RpcUrl, account, foundry, logger, l1Artifacts, {
    l2GasTokenAddress: GasTokenAddress,
    vkTreeRoot: getVKTreeRoot(),
  });

  return l1Data;
};

/**
 * Sets up Private eXecution Environment (PXE).
 * @param aztecNode - An instance of Aztec Node.
 * @param opts - Partial configuration for the PXE service.
 * @param firstPrivKey - The private key of the first account to be created.
 * @param logger - The logger to be used.
 * @param useLogSuffix - Whether to add a randomly generated suffix to the PXE debug logs.
 * @param proofCreator - An optional proof creator to use
 * @returns Private eXecution Environment (PXE), accounts, wallets and logger.
 */
export async function setupPXEService(
  aztecNode: AztecNode,
  opts: Partial<PXEServiceConfig> = {},
  logger = getLogger(),
  useLogSuffix = false,
  proofCreator?: BBNativePrivateKernelProver,
): Promise<{
  /**
   * The PXE instance.
   */
  pxe: PXEService;
  /**
   * Logger instance named as the current test.
   */
  logger: DebugLogger;
  /**
   * Teardown function
   */
  teardown: () => Promise<void>;
}> {
  const pxeServiceConfig = { ...getPXEServiceConfig(), ...opts };
  const pxe = await createPXEService(aztecNode, pxeServiceConfig, useLogSuffix, proofCreator);

  const teardown = async () => {
    await pxe.stop();
  };

  return {
    pxe,
    logger,
    teardown,
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
  enableGas: boolean,
) {
  // we are setting up against a remote environment, l1 contracts are already deployed
  const aztecNodeUrl = getAztecUrl();
  logger.verbose(`Creating Aztec Node client to remote host ${aztecNodeUrl}`);
  const aztecNode = createAztecNodeClient(aztecNodeUrl);
  logger.verbose(`Creating PXE client to remote host ${PXE_URL}`);
  const pxeClient = createPXEClient(PXE_URL, makeFetch([1, 2, 3], true));
  await waitForPXE(pxeClient, logger);
  logger.verbose('JSON RPC client connected to PXE');
  logger.verbose(`Retrieving contract addresses from ${PXE_URL}`);
  const l1Contracts = (await pxeClient.getNodeInfo()).l1ContractAddresses;

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

  const { chainId, protocolVersion } = await pxeClient.getNodeInfo();
  // this contract might already have been deployed
  // the following deploying functions are idempotent
  await deployCanonicalKeyRegistry(
    new SignerlessWallet(pxeClient, new DefaultMultiCallEntrypoint(chainId, protocolVersion)),
  );
  await deployCanonicalAuthRegistry(
    new SignerlessWallet(pxeClient, new DefaultMultiCallEntrypoint(config.chainId, config.version)),
  );

  if (enableGas) {
    await deployCanonicalGasToken(
      new SignerlessWallet(pxeClient, new DefaultMultiCallEntrypoint(chainId, protocolVersion)),
    );
  }

  logger.verbose('Constructing available wallets from already registered accounts...');
  const wallets = await getDeployedTestAccountsWallets(pxeClient);

  if (wallets.length < numberOfAccounts) {
    const numNewAccounts = numberOfAccounts - wallets.length;
    logger.verbose(`Deploying ${numNewAccounts} accounts...`);
    wallets.push(...(await createAccounts(pxeClient, numNewAccounts)));
  }

  return {
    aztecNode,
    sequencer: undefined,
    prover: undefined,
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
  /** The Aztec Node configuration. */
  config: AztecNodeConfig;
  /** The first wallet to be used. */
  wallet: AccountWalletWithSecretKey;
  /** The wallets to be used. */
  wallets: AccountWalletWithSecretKey[];
  /** Logger instance named as the current test. */
  logger: DebugLogger;
  /** The cheat codes. */
  cheatCodes: CheatCodes;
  /** Proving jobs */
  prover: ProverClient | undefined;
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
  enableGas = false,
): Promise<EndToEndContext> {
  const config = { ...getConfigEnvVars(), ...opts };
  const logger = getLogger();

  let anvil: Anvil | undefined;

  if (!config.rpcUrl) {
    if (PXE_URL) {
      throw new Error(
        `PXE_URL provided but no ETHEREUM_HOST set. Refusing to run, please set both variables so tests can deploy L1 contracts to the same Anvil instance`,
      );
    }

    const res = await startAnvil();
    anvil = res.anvil;
    config.rpcUrl = res.rpcUrl;
  }

  // Enable logging metrics to a local file named after the test suite
  if (isMetricsLoggingRequested()) {
    const filename = path.join('log', getJobName() + '.jsonl');
    logger.info(`Logging metrics to ${filename}`);
    setupMetricsLogger(filename);
  }

  if (opts.stateLoad) {
    const ethCheatCodes = new EthCheatCodes(config.rpcUrl);
    await ethCheatCodes.loadChainState(opts.stateLoad);
  }

  const hdAccount = mnemonicToAccount(MNEMONIC);
  const privKeyRaw = hdAccount.getHdKey().privateKey;
  const publisherPrivKey = privKeyRaw === null ? null : Buffer.from(privKeyRaw);

  if (PXE_URL) {
    // we are setting up against a remote environment, l1 contracts are assumed to already be deployed
    return await setupWithRemoteEnvironment(hdAccount, config, logger, numberOfAccounts, enableGas);
  }

  const deployL1ContractsValues =
    opts.deployL1ContractsValues ?? (await setupL1Contracts(config.rpcUrl, hdAccount, logger));

  config.publisherPrivateKey = `0x${publisherPrivKey!.toString('hex')}`;
  config.l1Contracts = deployL1ContractsValues.l1ContractAddresses;

  logger.verbose('Creating and synching an aztec node...');

  const acvmConfig = await getACVMConfig(logger);
  if (acvmConfig) {
    config.acvmWorkingDirectory = acvmConfig.acvmWorkingDirectory;
    config.acvmBinaryPath = acvmConfig.acvmBinaryPath;
  }

  const bbConfig = await getBBConfig(logger);
  if (bbConfig) {
    config.bbBinaryPath = bbConfig.bbBinaryPath;
    config.bbWorkingDirectory = bbConfig.bbWorkingDirectory;
  }
  config.l1BlockPublishRetryIntervalMS = 100;
  const aztecNode = await AztecNodeService.createAndSync(config, telemetry);
  const sequencer = aztecNode.getSequencer();
  const prover = aztecNode.getProver();

  logger.verbose('Creating a pxe...');

  const { pxe } = await setupPXEService(aztecNode!, pxeOpts, logger);

  logger.verbose('Deploying key registry...');
  await deployCanonicalKeyRegistry(
    new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(config.chainId, config.version)),
  );

  logger.verbose('Deploying auth registry...');
  await deployCanonicalAuthRegistry(
    new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(config.chainId, config.version)),
  );

  if (enableGas) {
    logger.verbose('Deploying gas token...');
    await deployCanonicalGasToken(
      new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(config.chainId, config.version)),
    );
  }

  const wallets = await createAccounts(pxe, numberOfAccounts);
  const cheatCodes = CheatCodes.create(config.rpcUrl, pxe!);

  const teardown = async () => {
    if (aztecNode instanceof AztecNodeService) {
      await aztecNode?.stop();
    }
    if (pxe instanceof PXEService) {
      await pxe?.stop();
    }

    if (acvmConfig?.cleanup) {
      // remove the temp directory created for the acvm
      logger.verbose(`Cleaning up ACVM state`);
      await acvmConfig.cleanup();
    }

    await anvil?.stop();
  };

  return {
    aztecNode,
    pxe,
    deployL1ContractsValues,
    config,
    wallet: wallets[0],
    wallets,
    logger,
    cheatCodes,
    sequencer,
    prover,
    teardown,
  };
}

/**
 * Ensures there's a running Anvil instance and returns the RPC URL.
 * @returns
 */
export async function startAnvil(): Promise<{ anvil: Anvil; rpcUrl: string }> {
  let rpcUrl: string | undefined = undefined;

  // Start anvil.
  // We go via a wrapper script to ensure if the parent dies, anvil dies.
  const anvil = await retry(
    async () => {
      const ethereumHostPort = await getPort();
      rpcUrl = `http://127.0.0.1:${ethereumHostPort}`;
      const anvil = createAnvil({ anvilBinary: './scripts/anvil_kill_wrapper.sh', port: ethereumHostPort });
      await anvil.start();
      return anvil;
    },
    'Start anvil',
    makeBackoff([5, 5, 5]),
  );

  if (!rpcUrl) {
    throw new Error('Failed to start anvil');
  }

  return { anvil, rpcUrl };
}
/**
 * Registers the contract class used for test accounts and publicly deploys the instances requested.
 * Use this when you need to make a public call to an account contract, such as for requesting a public authwit.
 * @param sender - Wallet to send the deployment tx.
 * @param accountsToDeploy - Which accounts to publicly deploy.
 */

// docs:start:public_deploy_accounts
export async function publicDeployAccounts(sender: Wallet, accountsToDeploy: Wallet[]) {
  const accountAddressesToDeploy = accountsToDeploy.map(a => a.getAddress());
  const instances = await Promise.all(accountAddressesToDeploy.map(account => sender.getContractInstance(account)));
  const batch = new BatchCall(sender, [
    (await registerContractClass(sender, SchnorrAccountContractArtifact)).request(),
    ...instances.map(instance => deployInstance(sender, instance!).request()),
  ]);
  await batch.send().wait();
}
// docs:end:public_deploy_accounts

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
export const expectsNumOfNoteEncryptedLogsInTheLastBlockToBe = async (
  aztecNode: AztecNode | undefined,
  numEncryptedLogs: number,
) => {
  if (!aztecNode) {
    // An api for retrieving encrypted logs does not exist on the PXE Service so we have to use the node
    // This means we can't perform this check if there is no node
    return;
  }
  const l2BlockNum = await aztecNode.getBlockNumber();
  const encryptedLogs = await aztecNode.getLogs(l2BlockNum, 1, LogType.NOTEENCRYPTED);
  const unrolledLogs = EncryptedNoteL2BlockL2Logs.unrollLogs(encryptedLogs);
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
): (...addresses: (AztecAddress | { address: AztecAddress })[]) => Promise<bigint[]> {
  const balances = async (...addressLikes: (AztecAddress | { address: AztecAddress })[]) => {
    const addresses = addressLikes.map(addressLike => ('address' in addressLike ? addressLike.address : addressLike));
    const b = await Promise.all(addresses.map(address => method(address).simulate()));
    const debugString = `${symbol} balances: ${addresses.map((address, i) => `${address}: ${b[i]}`).join(', ')}`;
    logger.verbose(debugString);
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

export async function expectMappingDelta<K, V extends number | bigint>(
  initialValues: V[],
  fn: (...k: K[]) => Promise<V[]>,
  inputs: K[],
  expectedDiffs: V[],
): Promise<void> {
  expect(inputs.length).toBe(expectedDiffs.length);

  const outputs = await fn(...inputs);
  const diffs = outputs.map((output, i) => output - initialValues[i]);

  expect(diffs).toEqual(expectedDiffs);
}

/**
 * Deploy the protocol contracts to a running instance.
 */
export async function deployCanonicalGasToken(pxe: PXE) {
  // "deploy" the Gas token as it contains public functions
  const gasPortalAddress = (await pxe.getNodeInfo()).l1ContractAddresses.gasPortalAddress;
  const canonicalGasToken = getCanonicalGasToken();

  if (await pxe.isContractClassPubliclyRegistered(canonicalGasToken.contractClass.id)) {
    getLogger().debug('Gas token already deployed');
    await expect(pxe.isContractPubliclyDeployed(canonicalGasToken.address)).resolves.toBe(true);
    return;
  }

  // Capsules will die soon, patience!
  const publicBytecode = canonicalGasToken.contractClass.packedBytecode;
  const encodedBytecode = bufferAsFields(publicBytecode, MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS);
  await pxe.addCapsule(encodedBytecode);

  await pxe.registerContract(canonicalGasToken);
  const wallet = new SignerlessWallet(pxe);
  const gasToken = await GasTokenContract.at(canonicalGasToken.address, wallet);

  await gasToken.methods
    .deploy(
      canonicalGasToken.contractClass.artifactHash,
      canonicalGasToken.contractClass.privateFunctionsRoot,
      canonicalGasToken.contractClass.publicBytecodeCommitment,
      gasPortalAddress,
    )
    .send({ fee: { paymentMethod: new NoFeePaymentMethod(), gasSettings: GasSettings.teardownless() } })
    .wait();

  getLogger().info(`Gas token publicly deployed at ${gasToken.address}`);

  await expect(pxe.isContractClassPubliclyRegistered(gasToken.instance.contractClassId)).resolves.toBe(true);
  await expect(pxe.getContractInstance(gasToken.address)).resolves.toBeDefined();
  await expect(pxe.isContractPubliclyDeployed(gasToken.address)).resolves.toBe(true);
}

export async function deployCanonicalKeyRegistry(deployer: Wallet) {
  const canonicalKeyRegistry = getCanonicalKeyRegistry();

  // We check to see if there exists a contract at the canonical Key Registry address with the same contract class id as we expect. This means that
  // the key registry has already been deployed to the correct address.
  if (
    (await deployer.getContractInstance(canonicalKeyRegistry.address))?.contractClassId.equals(
      canonicalKeyRegistry.contractClass.id,
    ) &&
    (await deployer.isContractClassPubliclyRegistered(canonicalKeyRegistry.contractClass.id))
  ) {
    return;
  }

  const keyRegistry = await KeyRegistryContract.deploy(deployer)
    .send({ contractAddressSalt: canonicalKeyRegistry.instance.salt, universalDeploy: true })
    .deployed();

  if (
    !keyRegistry.address.equals(canonicalKeyRegistry.address) ||
    !keyRegistry.address.equals(AztecAddress.fromBigInt(CANONICAL_KEY_REGISTRY_ADDRESS))
  ) {
    throw new Error(
      `Deployed Key Registry address ${keyRegistry.address} does not match expected address ${canonicalKeyRegistry.address}, or they both do not equal CANONICAL_KEY_REGISTRY_ADDRESS`,
    );
  }

  expect(computeContractAddressFromInstance(keyRegistry.instance)).toEqual(keyRegistry.address);
  expect(getContractClassFromArtifact(keyRegistry.artifact).id).toEqual(keyRegistry.instance.contractClassId);
  await expect(deployer.isContractClassPubliclyRegistered(canonicalKeyRegistry.contractClass.id)).resolves.toBe(true);
  await expect(deployer.getContractInstance(canonicalKeyRegistry.instance.address)).resolves.toBeDefined();
}

export async function deployCanonicalAuthRegistry(deployer: Wallet) {
  const canonicalAuthRegistry = getCanonicalAuthRegistry();

  // We check to see if there exists a contract at the canonical Auth Registry address with the same contract class id as we expect. This means that
  // the auth registry has already been deployed to the correct address.
  if (
    (await deployer.getContractInstance(canonicalAuthRegistry.address))?.contractClassId.equals(
      canonicalAuthRegistry.contractClass.id,
    ) &&
    (await deployer.isContractClassPubliclyRegistered(canonicalAuthRegistry.contractClass.id))
  ) {
    return;
  }

  const authRegistry = await AuthRegistryContract.deploy(deployer)
    .send({ contractAddressSalt: canonicalAuthRegistry.instance.salt, universalDeploy: true })
    .deployed();

  if (
    !authRegistry.address.equals(canonicalAuthRegistry.address) ||
    !authRegistry.address.equals(AztecAddress.fromBigInt(CANONICAL_AUTH_REGISTRY_ADDRESS))
  ) {
    throw new Error(
      `Deployed Auth Registry address ${authRegistry.address} does not match expected address ${canonicalAuthRegistry.address}, or they both do not equal CANONICAL_AUTH_REGISTRY_ADDRESS`,
    );
  }

  expect(computeContractAddressFromInstance(authRegistry.instance)).toEqual(authRegistry.address);
  expect(getContractClassFromArtifact(authRegistry.artifact).id).toEqual(authRegistry.instance.contractClassId);
  await expect(deployer.isContractClassPubliclyRegistered(canonicalAuthRegistry.contractClass.id)).resolves.toBe(true);
  await expect(deployer.getContractInstance(canonicalAuthRegistry.instance.address)).resolves.toBeDefined();
}
