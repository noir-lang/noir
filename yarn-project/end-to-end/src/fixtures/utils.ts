import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import {
  AccountWallet,
  AztecAddress,
  CheatCodes,
  CompleteAddress,
  EthAddress,
  EthCheatCodes,
  Wallet,
  createAccounts,
  createAztecRpcClient as createJsonRpcClient,
  getSandboxAccountsWallets,
} from '@aztec/aztec.js';
import { CircuitsWasm, GeneratorIndex } from '@aztec/circuits.js';
import { pedersenPlookupCompressWithHashIndex } from '@aztec/circuits.js/barretenberg';
import {
  DeployL1Contracts,
  L1ContractArtifactsForDeployment,
  deployL1Contract,
  deployL1Contracts,
} from '@aztec/ethereum';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import {
  ContractDeploymentEmitterAbi,
  ContractDeploymentEmitterBytecode,
  DecoderHelperAbi,
  DecoderHelperBytecode,
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
  TokenPortalAbi,
  TokenPortalBytecode,
} from '@aztec/l1-artifacts';
import { NonNativeTokenContract, TokenBridgeContract, TokenContract } from '@aztec/noir-contracts/types';
import { AztecRPCServer, createAztecRPCServer, getConfigEnvVars as getRpcConfigEnvVars } from '@aztec/pxe';
import { AztecRPC, L2BlockL2Logs, LogType, TxStatus } from '@aztec/types';

import {
  Account,
  Chain,
  HDAccount,
  HttpTransport,
  PrivateKeyAccount,
  PublicClient,
  WalletClient,
  createPublicClient,
  createWalletClient,
  getContract,
  http,
} from 'viem';
import { mnemonicToAccount } from 'viem/accounts';

import { MNEMONIC, localAnvil } from './fixtures.js';

const { SANDBOX_URL = '' } = process.env;

export const waitForRPCServer = async (rpcServer: AztecRPC, logger: DebugLogger) => {
  await retryUntil(async () => {
    try {
      logger('Attempting to contact RPC Server...');
      await rpcServer.getNodeInfo();
      return true;
    } catch (error) {
      logger('Failed to contact RPC Server!');
    }
    return undefined;
  }, 'RPC Get Node Info');
};

const createAztecNode = async (
  nodeConfig: AztecNodeConfig,
  logger: DebugLogger,
): Promise<AztecNodeService | undefined> => {
  if (SANDBOX_URL) {
    logger(`Not creating Aztec Node as we are running against a sandbox at ${SANDBOX_URL}`);
    return undefined;
  }
  logger('Creating and synching an aztec node...');
  return await AztecNodeService.createAndSync(nodeConfig);
};

export const setupL1Contracts = async (
  l1RpcUrl: string,
  account: HDAccount | PrivateKeyAccount,
  logger: DebugLogger,
  deployDecoderHelper = false,
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
    rollup: {
      contractAbi: RollupAbi,
      contractBytecode: RollupBytecode,
    },
  };
  if (deployDecoderHelper) {
    l1Artifacts.decoderHelper = {
      contractAbi: DecoderHelperAbi,
      contractBytecode: DecoderHelperBytecode,
    };
  }
  return await deployL1Contracts(l1RpcUrl, account, localAnvil, logger, l1Artifacts);
};

/**
 * Sets up Aztec RPC Server.
 * @param numberOfAccounts - The number of new accounts to be created once the RPC server is initiated.
 * @param aztecNode - The instance of an aztec node, if one is required
 * @param firstPrivKey - The private key of the first account to be created.
 * @param logger - The logger to be used.
 * @param useLogSuffix - Whether to add a randomly generated suffix to the RPC server debug logs.
 * @returns Aztec RPC server, accounts, wallets and logger.
 */
export async function setupAztecRPCServer(
  numberOfAccounts: number,
  aztecNode: AztecNodeService,
  logger = getLogger(),
  useLogSuffix = false,
): Promise<{
  /**
   * The Aztec RPC instance.
   */
  aztecRpcServer: AztecRPC;
  /**
   * The accounts created by the RPC server.
   */
  accounts: CompleteAddress[];
  /**
   * The wallets to be used.
   */
  wallets: AccountWallet[];
  /**
   * Logger instance named as the current test.
   */
  logger: DebugLogger;
}> {
  const rpcConfig = getRpcConfigEnvVars();
  const rpc = await createAztecRPCServer(aztecNode, rpcConfig, {}, useLogSuffix);

  const wallets = await createAccounts(rpc, numberOfAccounts);

  return {
    aztecRpcServer: rpc!,
    accounts: await rpc!.getRegisteredAccounts(),
    wallets,
    logger,
  };
}

/**
 * Function to setup the test against a running sandbox.
 * @param account - The account for use in create viem wallets.
 * @param config - The aztec Node Configuration
 * @param logger - The logger to be used
 * @returns RPC Client, viwm wallets, contract addreses etc.
 */
async function setupWithSandbox(account: Account, config: AztecNodeConfig, logger: DebugLogger) {
  // we are setting up against the sandbox, l1 contracts are already deployed
  logger(`Creating JSON RPC client to remote host ${SANDBOX_URL}`);
  const jsonClient = createJsonRpcClient(SANDBOX_URL);
  await waitForRPCServer(jsonClient, logger);
  logger('JSON RPC client connected to RPC Server');
  logger(`Retrieving contract addresses from ${SANDBOX_URL}`);
  const l1Contracts = (await jsonClient.getNodeInfo()).l1ContractAddresses;
  logger('RPC server created, constructing wallets from initial sandbox accounts...');
  const wallets = await getSandboxAccountsWallets(jsonClient);

  const walletClient = createWalletClient<HttpTransport, Chain, HDAccount>({
    account,
    chain: localAnvil,
    transport: http(config.rpcUrl),
  });
  const publicClient = createPublicClient({
    chain: localAnvil,
    transport: http(config.rpcUrl),
  });
  const deployL1ContractsValues: DeployL1Contracts = {
    l1ContractAddresses: l1Contracts,
    walletClient,
    publicClient,
  };
  const cheatCodes = await CheatCodes.create(config.rpcUrl, jsonClient!);
  const teardown = () => Promise.resolve();
  return {
    aztecNode: undefined,
    aztecRpcServer: jsonClient,
    deployL1ContractsValues,
    accounts: await jsonClient!.getRegisteredAccounts(),
    config,
    wallet: wallets[0],
    wallets,
    logger,
    cheatCodes,
    teardown,
  };
}

/**
 * Sets up the environment for the end-to-end tests.
 * @param numberOfAccounts - The number of new accounts to be created once the RPC server is initiated.
 */
export async function setup(
  numberOfAccounts = 1,
  stateLoad: string | undefined = undefined,
): Promise<{
  /**
   * The Aztec Node service.
   */
  aztecNode: AztecNodeService | undefined;
  /**
   * The Aztec RPC server.
   */
  aztecRpcServer: AztecRPC;
  /**
   * Return values from deployL1Contracts function.
   */
  deployL1ContractsValues: DeployL1Contracts;
  /**
   * The accounts created by the RPC server.
   */
  accounts: CompleteAddress[];
  /**
   * The Aztec Node configuration.
   */
  config: AztecNodeConfig;
  /**
   * The first wallet to be used.
   */
  wallet: AccountWallet;
  /**
   * The wallets to be used.
   */
  wallets: AccountWallet[];
  /**
   * Logger instance named as the current test.
   */
  logger: DebugLogger;
  /**
   * The cheat codes.
   */
  cheatCodes: CheatCodes;
  /**
   * Function to stop the started services.
   */
  teardown: () => Promise<void>;
}> {
  const config = getConfigEnvVars();

  if (stateLoad) {
    const ethCheatCodes = new EthCheatCodes(config.rpcUrl);
    await ethCheatCodes.loadChainState(stateLoad);
  }

  const logger = getLogger();
  const hdAccount = mnemonicToAccount(MNEMONIC);

  if (SANDBOX_URL) {
    // we are setting up against the sandbox, l1 contracts are already deployed
    return await setupWithSandbox(hdAccount, config, logger);
  }

  const deployL1ContractsValues = await setupL1Contracts(config.rpcUrl, hdAccount, logger);
  const privKeyRaw = hdAccount.getHdKey().privateKey;
  const publisherPrivKey = privKeyRaw === null ? null : Buffer.from(privKeyRaw);

  config.publisherPrivateKey = `0x${publisherPrivKey!.toString('hex')}`;
  config.l1Contracts.rollupAddress = deployL1ContractsValues.l1ContractAddresses.rollupAddress;
  config.l1Contracts.registryAddress = deployL1ContractsValues.l1ContractAddresses.registryAddress;
  config.l1Contracts.contractDeploymentEmitterAddress =
    deployL1ContractsValues.l1ContractAddresses.contractDeploymentEmitterAddress;
  config.l1Contracts.inboxAddress = deployL1ContractsValues.l1ContractAddresses.inboxAddress;

  const aztecNode = await createAztecNode(config, logger);

  const { aztecRpcServer, accounts, wallets } = await setupAztecRPCServer(numberOfAccounts, aztecNode!, logger);

  const cheatCodes = await CheatCodes.create(config.rpcUrl, aztecRpcServer!);

  const teardown = async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) await aztecRpcServer?.stop();
  };

  return {
    aztecNode,
    aztecRpcServer,
    deployL1ContractsValues,
    accounts,
    config,
    wallet: wallets[0],
    wallets,
    logger,
    cheatCodes,
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

/**
 * Returns a logger instance for the current test.
 * @returns a logger instance for the current test.
 */
export function getLogger() {
  const describeBlockName = expect.getState().currentTestName?.split(' ')[0];
  return createDebugLogger('aztec:' + describeBlockName);
}

/**
 * Deploy L1 token and portal, initialize portal, deploy a non native l2 token contract, its L2 bridge contract and attach is to the portal.
 * @param wallet - the wallet instance
 * @param walletClient - A viem WalletClient.
 * @param publicClient - A viem PublicClient.
 * @param rollupRegistryAddress - address of rollup registry to pass to initialize the token portal
 * @param owner - owner of the L2 contract
 * @param underlyingERC20Address - address of the underlying ERC20 contract to use (if none supplied, it deploys one)
 * @returns l2 contract instance, bridge contract instance, token portal instance, token portal address and the underlying ERC20 instance
 */
export async function deployAndInitializeStandardizedTokenAndBridgeContracts(
  wallet: Wallet,
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  rollupRegistryAddress: EthAddress,
  owner: AztecAddress,
  underlyingERC20Address?: EthAddress,
): Promise<{
  /**
   * The L2 token contract instance.
   */
  token: TokenContract;
  /**
   * The L2 bridge contract instance.
   */
  bridge: TokenBridgeContract;
  /**
   * The token portal contract address.
   */
  tokenPortalAddress: EthAddress;
  /**
   * The token portal contract instance
   */
  tokenPortal: any;
  /**
   * The underlying ERC20 contract instance.
   */
  underlyingERC20: any;
}> {
  if (!underlyingERC20Address) {
    underlyingERC20Address = await deployL1Contract(walletClient, publicClient, PortalERC20Abi, PortalERC20Bytecode);
  }
  const underlyingERC20 = getContract({
    address: underlyingERC20Address.toString(),
    abi: PortalERC20Abi,
    walletClient,
    publicClient,
  });

  // deploy the token portal
  const tokenPortalAddress = await deployL1Contract(walletClient, publicClient, TokenPortalAbi, TokenPortalBytecode);
  const tokenPortal = getContract({
    address: tokenPortalAddress.toString(),
    abi: TokenPortalAbi,
    walletClient,
    publicClient,
  });

  // deploy l2 token
  const deployTx = TokenContract.deploy(wallet).send();

  // deploy l2 token bridge and attach to the portal
  const bridgeTx = TokenBridgeContract.deploy(wallet).send({
    portalContract: tokenPortalAddress,
    contractAddressSalt: Fr.random(),
  });

  // now wait for the deploy txs to be mined. This way we send all tx in the same rollup.
  const deployReceipt = await deployTx.wait();
  if (deployReceipt.status !== TxStatus.MINED) throw new Error(`Deploy token tx status is ${deployReceipt.status}`);
  const token = await TokenContract.at(deployReceipt.contractAddress!, wallet);

  const bridgeReceipt = await bridgeTx.wait();
  if (bridgeReceipt.status !== TxStatus.MINED) throw new Error(`Deploy bridge tx status is ${bridgeReceipt.status}`);
  const bridge = await TokenBridgeContract.at(bridgeReceipt.contractAddress!, wallet);
  await bridge.attach(tokenPortalAddress);
  const bridgeAddress = bridge.address.toString() as `0x${string}`;

  // initialize l2 token
  const initializeTx = token.methods._initialize(owner).send();

  // initialize bridge
  const initializeBridgeTx = bridge.methods._initialize(token.address).send();

  // now we wait for the txs to be mined. This way we send all tx in the same rollup.
  const initializeReceipt = await initializeTx.wait();
  if (initializeReceipt.status !== TxStatus.MINED)
    throw new Error(`Initialize token tx status is ${initializeReceipt.status}`);
  if ((await token.methods.admin().view()) !== owner.toBigInt()) throw new Error(`Token admin is not ${owner}`);

  const initializeBridgeReceipt = await initializeBridgeTx.wait();
  if (initializeBridgeReceipt.status !== TxStatus.MINED)
    throw new Error(`Initialize token bridge tx status is ${initializeBridgeReceipt.status}`);
  if ((await bridge.methods.token().view()) !== token.address.toBigInt())
    throw new Error(`Bridge token is not ${token.address}`);

  // make the bridge a minter on the token:
  const makeMinterTx = token.methods.set_minter(bridge.address, true).send();
  const makeMinterReceipt = await makeMinterTx.wait();
  if (makeMinterReceipt.status !== TxStatus.MINED)
    throw new Error(`Make bridge a minter tx status is ${makeMinterReceipt.status}`);
  if ((await token.methods.is_minter(bridge.address).view()) === 1n) throw new Error(`Bridge is not a minter`);

  // initialize portal
  await tokenPortal.write.initialize(
    [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), bridgeAddress],
    {} as any,
  );

  return { token, bridge, tokenPortalAddress, tokenPortal, underlyingERC20 };
}

/**
 * Deploy L1 token and portal, initialize portal, deploy a non native l2 token contract and attach is to the portal.
 * @param aztecRpcServer - the aztec rpc server instance
 * @param walletClient - A viem WalletClient.
 * @param publicClient - A viem PublicClient.
 * @param rollupRegistryAddress - address of rollup registry to pass to initialize the token portal
 * @param initialBalance - initial balance of the owner of the L2 contract
 * @param owner - owner of the L2 contract
 * @param underlyingERC20Address - address of the underlying ERC20 contract to use (if none supplied, it deploys one)
 * @returns l2 contract instance, token portal instance, token portal address and the underlying ERC20 instance
 */
// TODO (#2291) DELETE!!!
export async function deployAndInitializeNonNativeL2TokenContracts(
  wallet: Wallet,
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  rollupRegistryAddress: EthAddress,
  initialBalance = 0n,
  owner = AztecAddress.ZERO,
  underlyingERC20Address?: EthAddress,
) {
  // deploy underlying contract if no address supplied
  if (!underlyingERC20Address) {
    underlyingERC20Address = await deployL1Contract(walletClient, publicClient, PortalERC20Abi, PortalERC20Bytecode);
  }
  const underlyingERC20: any = getContract({
    address: underlyingERC20Address.toString(),
    abi: PortalERC20Abi,
    walletClient,
    publicClient,
  });

  // deploy the token portal
  const tokenPortalAddress = await deployL1Contract(walletClient, publicClient, TokenPortalAbi, TokenPortalBytecode);
  const tokenPortal: any = getContract({
    address: tokenPortalAddress.toString(),
    abi: TokenPortalAbi,
    walletClient,
    publicClient,
  });

  // deploy l2 contract and attach to portal
  const tx = NonNativeTokenContract.deploy(wallet, initialBalance, owner).send({
    portalContract: tokenPortalAddress,
    contractAddressSalt: Fr.random(),
  });
  await tx.isMined({ interval: 0.1 });
  const receipt = await tx.getReceipt();
  if (receipt.status !== TxStatus.MINED) throw new Error(`Tx status is ${receipt.status}`);
  const l2Contract = await NonNativeTokenContract.at(receipt.contractAddress!, wallet);
  await l2Contract.attach(tokenPortalAddress);
  const l2TokenAddress = l2Contract.address.toString() as `0x${string}`;

  // initialize portal
  await tokenPortal.write.initialize(
    [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), l2TokenAddress],
    {} as any,
  );
  return { l2Contract, tokenPortalAddress, tokenPortal, underlyingERC20 };
}

/**
 * Sleep for a given number of milliseconds.
 * @param ms - the number of milliseconds to sleep for
 */
export function delay(ms: number): Promise<void> {
  return new Promise<void>(resolve => setTimeout(resolve, ms));
}

/**
 * Checks the number of encrypted logs in the last block is as expected.
 * @param aztecNode - The instance of aztec node for retrieving the logs.
 * @param numEncryptedLogs - The number of expected logs.
 */
export const expectsNumOfEncryptedLogsInTheLastBlockToBe = async (
  aztecNode: AztecNodeService | undefined,
  numEncryptedLogs: number,
) => {
  if (!aztecNode) {
    // An api for retrieving encrypted logs does not exist on the rpc server so we have to use the node
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
 * @param rpc - The instance of AztecRPC for retrieving the logs.
 * @param logMessages - The set of expected log messages.
 */
export const expectUnencryptedLogsFromLastBlockToBe = async (rpc: AztecRPC, logMessages: string[]) => {
  // docs:start:get_logs
  // Get the latest block number to retrieve logs from
  const l2BlockNum = await rpc.getBlockNumber();
  // Get the unencrypted logs from the last block
  const unencryptedLogs = await rpc.getUnencryptedLogs(l2BlockNum, 1);
  // docs:end:get_logs
  const unrolledLogs = L2BlockL2Logs.unrollLogs(unencryptedLogs);
  const asciiLogs = unrolledLogs.map(log => log.toString('ascii'));

  expect(asciiLogs).toStrictEqual(logMessages);
};

/**
 * Hash a payload to generate a signature on an account contract
 * @param payload - payload to hash
 * @returns the hashed message
 */
export const hashPayload = async (payload: Fr[]) => {
  return pedersenPlookupCompressWithHashIndex(
    await CircuitsWasm.get(),
    payload.map(fr => fr.toBuffer()),
    GeneratorIndex.SIGNATURE_PAYLOAD,
  );
};
