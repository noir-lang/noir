import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { RpcServerConfig, createAztecRPCServer, getConfigEnvVars as getRpcConfigEnvVars } from '@aztec/aztec-rpc';
import {
  Account as AztecAccount,
  AztecAddress,
  CheatCodes,
  Contract,
  ContractDeployer,
  EntrypointCollection,
  EntrypointWallet,
  EthAddress,
  L1CheatCodes,
  Wallet,
  createAztecRpcClient as createJsonRpcClient,
  getL1ContractAddresses,
  getSandboxAccountsWallet,
  getUnsafeSchnorrAccount,
} from '@aztec/aztec.js';
import { CompleteAddress, PrivateKey, PublicKey } from '@aztec/circuits.js';
import { DeployL1Contracts, deployL1Contract, deployL1Contracts } from '@aztec/ethereum';
import { ContractAbi } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { mustSucceedFetch } from '@aztec/foundation/json-rpc/client';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';
import { NonNativeTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, L2BlockL2Logs, LogType, TxStatus } from '@aztec/types';

import every from 'lodash.every';
import zipWith from 'lodash.zipwith';
import {
  Account,
  Chain,
  HDAccount,
  HttpTransport,
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

const createRpcServer = async (
  rpcConfig: RpcServerConfig,
  aztecNode: AztecNodeService | undefined,
  logger: DebugLogger,
  useLogSuffix?: boolean | string,
): Promise<AztecRPC> => {
  if (SANDBOX_URL) {
    logger(`Creating JSON RPC client to remote host ${SANDBOX_URL}`);
    const jsonClient = createJsonRpcClient(SANDBOX_URL, mustSucceedFetch);
    await waitForRPCServer(jsonClient, logger);
    logger('JSON RPC client connected to RPC Server');
    return jsonClient;
  } else if (!aztecNode) {
    throw new Error('Invalid aztec node when creating RPC server');
  }
  return createAztecRPCServer(aztecNode, rpcConfig, {}, useLogSuffix);
};

const setupL1Contracts = async (l1RpcUrl: string, account: HDAccount, logger: DebugLogger) => {
  if (SANDBOX_URL) {
    logger(`Retrieving contract addresses from ${SANDBOX_URL}`);
    const l1Contracts = await getL1ContractAddresses(SANDBOX_URL);

    const walletClient = createWalletClient<HttpTransport, Chain, HDAccount>({
      account,
      chain: localAnvil,
      transport: http(l1RpcUrl),
    });
    const publicClient = createPublicClient({
      chain: localAnvil,
      transport: http(l1RpcUrl),
    });
    return {
      rollupAddress: l1Contracts.rollup,
      registryAddress: l1Contracts.registry,
      inboxAddress: l1Contracts.inbox,
      outboxAddress: l1Contracts.outbox,
      contractDeploymentEmitterAddress: l1Contracts.contractDeploymentEmitter,
      decoderHelperAddress: l1Contracts.decoderHelper,
      walletClient,
      publicClient,
    };
  }
  return await deployL1Contracts(l1RpcUrl, account, localAnvil, logger);
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
  aztecNode: AztecNodeService | undefined,
  firstPrivKey: PrivateKey | null = null,
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
   * The wallet to be used.
   */
  wallet: Wallet;
  /**
   * Logger instance named as the current test.
   */
  logger: DebugLogger;
}> {
  const rpcConfig = getRpcConfigEnvVars();
  const aztecRpcServer = await createRpcServer(rpcConfig, aztecNode, logger, useLogSuffix);

  const accounts: AztecAccount[] = [];

  const createWalletWithAccounts = async () => {
    if (!SANDBOX_URL) {
      logger('RPC server created, deploying new accounts...');

      // Prepare deployments
      for (let i = 0; i < numberOfAccounts; ++i) {
        const privateKey = i === 0 && firstPrivKey !== null ? firstPrivKey! : PrivateKey.random();
        const account = getUnsafeSchnorrAccount(aztecRpcServer, privateKey);
        await account.getDeployMethod().then(d => d.simulate({ contractAddressSalt: account.salt }));
        accounts.push(account);
      }

      // Send them and await them to be mined
      const txs = await Promise.all(accounts.map(account => account.deploy()));
      await Promise.all(txs.map(tx => tx.wait({ interval: 0.1 })));
      return new EntrypointWallet(aztecRpcServer, await EntrypointCollection.fromAccounts(accounts));
    } else {
      logger('RPC server created, constructing wallet from initial sandbox accounts...');
      return await getSandboxAccountsWallet(aztecRpcServer);
    }
  };

  const wallet = await createWalletWithAccounts();

  return {
    aztecRpcServer: aztecRpcServer!,
    accounts: await aztecRpcServer!.getAccounts(),
    wallet,
    logger,
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
   * The wallet to be used.
   */
  wallet: Wallet;
  /**
   * Logger instance named as the current test.
   */
  logger: DebugLogger;
  /**
   * The cheat codes.
   */
  cheatCodes: CheatCodes;
}> {
  const config = getConfigEnvVars();

  if (stateLoad) {
    const l1CheatCodes = new L1CheatCodes(config.rpcUrl);
    await l1CheatCodes.loadChainState(stateLoad);
  }

  const logger = getLogger();
  const hdAccount = mnemonicToAccount(MNEMONIC);

  const deployL1ContractsValues = await setupL1Contracts(config.rpcUrl, hdAccount, logger);
  const privKeyRaw = hdAccount.getHdKey().privateKey;
  const privKey = privKeyRaw === null ? null : new PrivateKey(Buffer.from(privKeyRaw));

  config.publisherPrivateKey = privKey!;
  config.rollupContract = deployL1ContractsValues.rollupAddress;
  config.contractDeploymentEmitterContract = deployL1ContractsValues.contractDeploymentEmitterAddress;
  config.inboxContract = deployL1ContractsValues.inboxAddress;

  const aztecNode = await createAztecNode(config, logger);

  const { aztecRpcServer, accounts, wallet } = await setupAztecRPCServer(numberOfAccounts, aztecNode, privKey, logger);

  const cheatCodes = await CheatCodes.create(config.rpcUrl, aztecRpcServer!);

  return {
    aztecNode,
    aztecRpcServer,
    deployL1ContractsValues,
    accounts,
    config,
    wallet,
    logger,
    cheatCodes,
  };
}

/**
 * Deploys a smart contract on L2.
 * @param aztecRpcServer - An instance of AztecRPC that will be used for contract deployment.
 * @param publicKey - The encryption public key.
 * @param abi - The Contract ABI (Application Binary Interface) that defines the contract's interface.
 * @param args - An array of arguments to be passed to the contract constructor during deployment.
 * @param contractAddressSalt - A random value used as a salt to generate the contract address. If not provided, the contract address will be deterministic.
 * @returns An object containing the deployed contract's address and partial address.
 */
export async function deployContract(
  aztecRpcServer: AztecRPC,
  publicKey: PublicKey,
  abi: ContractAbi,
  args: any[],
  contractAddressSalt?: Fr,
) {
  const deployer = new ContractDeployer(abi, aztecRpcServer, publicKey);
  const deployMethod = deployer.deploy(...args);
  await deployMethod.create({ contractAddressSalt });
  const tx = deployMethod.send();
  expect(await tx.isMined({ interval: 0.1 })).toBeTruthy();
  const receipt = await tx.getReceipt();
  return { address: receipt.contractAddress!, partialAddress: deployMethod.partialAddress! };
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
 * Deploys a set of contracts to the network.
 * @param wallet - the wallet to make the request.
 * @param abi - contracts to be deployed.
 * @returns The deployed contract instances.
 */
export async function deployL2Contracts(wallet: Wallet, abis: ContractAbi[]) {
  const logger = getLogger();
  const calls = await Promise.all(abis.map(abi => new ContractDeployer(abi, wallet).deploy()));
  for (const call of calls) await call.create();
  const txs = await Promise.all(calls.map(c => c.send()));
  expect(every(await Promise.all(txs.map(tx => tx.isMined({ interval: 0.1 }))))).toBeTruthy();
  const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
  const contracts = zipWith(
    abis,
    receipts,
    async (abi, receipt) => await Contract.at(receipt!.contractAddress!, abi!, wallet),
  );
  contracts.forEach(async c => logger(`L2 contract ${(await c).abi.name} deployed at ${(await c).address}`));
  return contracts;
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
  const l2BlockNum = await aztecNode.getBlockHeight();
  const encryptedLogs = await aztecNode.getLogs(l2BlockNum, 1, LogType.ENCRYPTED);
  const unrolledLogs = L2BlockL2Logs.unrollLogs(encryptedLogs);
  expect(unrolledLogs.length).toBe(numEncryptedLogs);
};

/**
 * Checks that the last block contains the given expected unencrypted log messages.
 * @param aztecNode - The instance of aztec node for retrieving the logs.
 * @param logMessages - The set of expected log messages.
 * @returns
 */
export const expectUnencryptedLogsFromLastBlockToBe = async (
  aztecNode: AztecNodeService | undefined,
  logMessages: string[],
) => {
  if (!aztecNode) {
    // An api for retrieving encrypted logs does not exist on the rpc server so we have to use the node
    // This means we can't perform this check if there is no node
    return;
  }
  const l2BlockNum = await aztecNode.getBlockHeight();
  const unencryptedLogs = await aztecNode.getLogs(l2BlockNum, 1, LogType.UNENCRYPTED);
  const unrolledLogs = L2BlockL2Logs.unrollLogs(unencryptedLogs);
  const asciiLogs = unrolledLogs.map(log => log.toString('ascii'));

  expect(asciiLogs).toStrictEqual(logMessages);
};
