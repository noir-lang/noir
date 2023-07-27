import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { RpcServerConfig, createAztecRPCServer, getConfigEnvVars as getRpcConfigEnvVars } from '@aztec/aztec-rpc';
import {
  AccountCollection,
  AccountImplementation,
  AccountWallet,
  AztecAddress,
  Contract,
  ContractDeployer,
  DeployMethod,
  EthAddress,
  SentTx,
  SingleKeyAccountContract,
  Wallet,
  createAztecRpcClient as createJsonRpcClient,
  generatePublicKey,
  getL1ContractAddresses,
} from '@aztec/aztec.js';
import {
  CircuitsWasm,
  DeploymentInfo,
  PartialContractAddress,
  PrivateKey,
  PublicKey,
  getContractDeploymentInfo,
} from '@aztec/circuits.js';
import { Schnorr, pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { DeployL1Contracts, deployL1Contract, deployL1Contracts } from '@aztec/ethereum';
import { ContractAbi } from '@aztec/foundation/abi';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { mustSucceedFetch } from '@aztec/foundation/json-rpc';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';
import { SchnorrSingleKeyAccountContractAbi } from '@aztec/noir-contracts/artifacts';
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

const waitForRPCServer = async (rpcServer: AztecRPC, logger: DebugLogger) => {
  await retryUntil(async () => {
    try {
      logger('Attmpting to contact RPC Server...');
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
 * Container to hold information about txs
 */
type TxContext = {
  /**
   * The fully built and sent transaction.
   */
  tx: SentTx | undefined;
  /**
   * The deploy method.
   */
  deployMethod: DeployMethod;

  /**
   * Contract address salt.
   */
  salt: Fr;

  /**
   * The user's private key.
   */
  privateKey: PrivateKey;
  /**
   * The fully derived deployment data.
   */
  deploymentData: DeploymentInfo;
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
  accounts: AztecAddress[];
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

  const accountCollection = new AccountCollection();
  const txContexts: TxContext[] = [];

  logger('RPC server created, deploying accounts...');

  for (let i = 0; i < numberOfAccounts; ++i) {
    // We use the well-known private key and the validating account contract for the first account,
    // and generate random key pairs for the rest.
    // TODO(#662): Let the aztec rpc server generate the key pair rather than hardcoding the private key
    const privateKey = i === 0 && firstPrivKey !== null ? firstPrivKey! : PrivateKey.random();
    const publicKey = await generatePublicKey(privateKey);
    const salt = Fr.random();
    const deploymentData = await getContractDeploymentInfo(SchnorrSingleKeyAccountContractAbi, [], salt, publicKey);

    const contractDeployer = new ContractDeployer(SchnorrSingleKeyAccountContractAbi, aztecRpcServer!, publicKey);
    const deployMethod = contractDeployer.deploy();
    await deployMethod.simulate({ contractAddressSalt: salt });
    txContexts.push({
      tx: undefined,
      deployMethod,
      salt,
      privateKey,
      deploymentData,
    });
  }

  // We do this in a separate loop to try and get all transactions into the same rollup.
  // Doing this here will submit the transactions with minimal delay between them.
  for (const context of txContexts) {
    logger(`Deploying account contract for ${context.deploymentData.address.toString()}`);
    context.tx = context.deployMethod.send();
  }

  for (const context of txContexts) {
    const publicKey = await generatePublicKey(context.privateKey);
    await context.tx!.isMined(0, 0.1);
    const receipt = await context.tx!.getReceipt();
    if (receipt.status !== TxStatus.MINED) {
      throw new Error(`Deployment tx not mined (status is ${receipt.status})`);
    }
    const receiptAddress = receipt.contractAddress!;
    if (!receiptAddress.equals(context.deploymentData.address)) {
      throw new Error(
        `Deployment address does not match for account contract (expected ${context.deploymentData.address} got ${receiptAddress})`,
      );
    }
    await aztecRpcServer!.addAccount(
      context.privateKey,
      context.deploymentData.address,
      context.deploymentData.partialAddress,
    );
    accountCollection.registerAccount(
      context.deploymentData.address,
      new SingleKeyAccountContract(
        context.deploymentData.address,
        context.deploymentData.partialAddress,
        context.privateKey,
        await Schnorr.new(),
      ),
    );
    logger(`Created account ${context.deploymentData.address.toString()} with public key ${publicKey.toString()}`);
  }

  const accounts = await aztecRpcServer!.getAccounts();
  const wallet = new AccountWallet(aztecRpcServer!, accountCollection);

  return {
    aztecRpcServer: aztecRpcServer!,
    accounts,
    wallet,
    logger,
  };
}

/**
 * Sets up the environment for the end-to-end tests.
 * @param numberOfAccounts - The number of new accounts to be created once the RPC server is initiated.
 */
export async function setup(numberOfAccounts = 1): Promise<{
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
  accounts: AztecAddress[];
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
}> {
  const config = getConfigEnvVars();
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

  return {
    aztecNode,
    aztecRpcServer,
    deployL1ContractsValues,
    accounts,
    config,
    wallet,
    logger,
  };
}

/**
 * Deploys a smart contract on L2.
 * @param aztecRpcServer - An instance of AztecRPC that will be used for contract deployment.
 * @param publicKey - The encryption public key.
 * @param abi - The Contract ABI (Application Binary Interface) that defines the contract's interface.
 * @param args - An array of arguments to be passed to the contract constructor during deployment.
 * @param contractAddressSalt - A random value used as a salt to generate the contract address. If not provided, the contract address will be deterministic.
 * @returns An object containing the deployed contract's address and partial contract address.
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
  expect(await tx.isMined(0, 0.1)).toBeTruthy();
  const receipt = await tx.getReceipt();
  return { address: receipt.contractAddress!, partialContractAddress: deployMethod.partialContractAddress! };
}

/**
 * Represents a function that creates an AccountImplementation object asynchronously.
 *
 * @param address - The Aztec address associated with the account.
 * @param useProperKey - A flag indicating whether the proper key should be used during account creation.
 * @param partialAddress - The partial contract address associated with the account.
 * @param encryptionPrivateKey - The encryption private key used during account creation.
 * @returns A Promise that resolves to an AccountImplementation object.
 */
export type CreateAccountImplFn = (
  address: AztecAddress,
  useProperKey: boolean,
  partialAddress: PartialContractAddress,
  encryptionPrivateKey: PrivateKey,
) => Promise<AccountImplementation>;

/**
 * Creates a new account.
 * @param aztecRpcServer - The AztecRPC server to interact with.
 * @param abi - The ABI (Application Binary Interface) of the account contract.
 * @param args - The arguments to pass to the account contract's constructor.
 * @param encryptionPrivateKey - The encryption private key used by the account.
 * @param useProperKey - A flag indicating whether the proper key should be used during account creation.
 * @param createAccountImpl - A function that creates an AccountImplementation object.
 * @returns A Promise that resolves to an object containing the created wallet, account address, and partial address.
 */
export async function createNewAccount(
  aztecRpcServer: AztecRPC,
  abi: ContractAbi,
  args: any[],
  encryptionPrivateKey: PrivateKey,
  useProperKey: boolean,
  createAccountImpl: CreateAccountImplFn,
) {
  const salt = Fr.random();
  const publicKey = await generatePublicKey(encryptionPrivateKey);
  const { address, partialAddress } = await getContractDeploymentInfo(abi, args, salt, publicKey);
  await aztecRpcServer.addAccount(encryptionPrivateKey, address, partialAddress);
  await deployContract(aztecRpcServer, publicKey, abi, args, salt);
  const account = await createAccountImpl(address, useProperKey, partialAddress, encryptionPrivateKey);
  const wallet = new AccountWallet(aztecRpcServer, account);
  return { wallet, address, partialAddress };
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
  expect(every(await Promise.all(txs.map(tx => tx.isMined(0, 0.1))))).toBeTruthy();
  const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
  const contracts = zipWith(abis, receipts, (abi, receipt) => new Contract(receipt!.contractAddress!, abi!, wallet));
  contracts.forEach(c => logger(`L2 contract ${c.abi.name} deployed at ${c.address}`));
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
 * @param underlyingERC20Address - address of the underlying ERC20 contract to use (if noone supplied, it deploys one)
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
  await tx.isMined(0, 0.1);
  const receipt = await tx.getReceipt();
  if (receipt.status !== TxStatus.MINED) throw new Error(`Tx status is ${receipt.status}`);
  const l2Contract = new NonNativeTokenContract(receipt.contractAddress!, wallet);
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
 * Calculates the slot value of a mapping within noir.
 * @param slot - The storage slot of the mapping.
 * @param key - The key within the mapping.
 * @returns The mapping's key.
 */
export async function calculateAztecStorageSlot(slot: bigint, key: Fr): Promise<Fr> {
  const wasm = await CircuitsWasm.get();
  const mappingStorageSlot = new Fr(slot); // this value is manually set in the Noir contract

  // Based on `at` function in
  // aztec3-packages/yarn-project/noir-contracts/src/contracts/noir-aztec/src/state_vars/map.nr
  const storageSlot = Fr.fromBuffer(
    pedersenPlookupCommitInputs(
      wasm,
      [mappingStorageSlot, key].map(f => f.toBuffer()),
    ),
  );

  return storageSlot; //.value;
}

/**
 * Check the value of a public mapping's storage slot.
 * @param logger - A logger instance.
 * @param aztecNode - An instance of the aztec node service.
 * @param contract - The contract to check the storage slot of.
 * @param slot - The mapping's storage slot.
 * @param key - The mapping's key.
 * @param expectedValue - The expected value of the mapping.
 */
export async function expectAztecStorageSlot(
  logger: DebugLogger,
  aztecRpc: AztecRPC,
  contract: Contract,
  slot: bigint,
  key: Fr,
  expectedValue: bigint,
) {
  const storageSlot = await calculateAztecStorageSlot(slot, key);
  const storageValue = await aztecRpc.getPublicStorageAt(contract.address!, storageSlot);
  if (storageValue === undefined) {
    throw new Error(`Storage slot ${storageSlot} not found`);
  }

  const balance = toBigIntBE(storageValue);

  logger(`Account ${key.toShortString()} balance: ${balance}`);
  expect(balance).toBe(expectedValue);
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
