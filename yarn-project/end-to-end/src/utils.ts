import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, Logger, createDebugLogger } from '@aztec/foundation/log';
import { MAPPING_SLOT_PEDERSEN_SEPARATOR } from '@aztec/circuits.js';

import {
  AccountCollection,
  AccountContract,
  AccountWallet,
  AztecAddress,
  AztecRPCServer,
  Contract,
  ContractDeployer,
  EthAddress,
  Point,
  TxStatus,
  Wallet,
  createAztecRPCServer,
  generatePublicKey,
  getContractDeploymentInfo,
} from '@aztec/aztec.js';
import { CircuitsWasm } from '@aztec/circuits.js';
import { Schnorr, pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { DeployL1Contracts, deployL1Contract, deployL1Contracts } from '@aztec/ethereum';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';
import { NonNativeTokenContractAbi, SchnorrAccountContractAbi } from '@aztec/noir-contracts/examples';
import { randomBytes } from 'crypto';
import { Account, Chain, HttpTransport, PublicClient, WalletClient, getContract } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { SchnorrAuthProvider } from './auth.js';
import { MNEMONIC, localAnvil } from './fixtures.js';
import every from 'lodash.every';
import zipWith from 'lodash.zipwith';
import { ContractAbi } from '@aztec/foundation/abi';

/**
 * Sets up the environment for the end-to-end tests.
 * @param numberOfAccounts - The number of new accounts to be created once the RPC server is initiated.
 */
export async function setup(numberOfAccounts = 1): Promise<{
  /**
   * The Aztec Node service.
   */
  aztecNode: AztecNodeService;
  /**
   * The Aztec RPC server.
   */
  aztecRpcServer: AztecRPCServer;
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
  const privKey = hdAccount.getHdKey().privateKey;
  const deployL1ContractsValues = await deployL1Contracts(config.rpcUrl, hdAccount, localAnvil, logger);

  config.publisherPrivateKey = Buffer.from(privKey!);
  config.rollupContract = deployL1ContractsValues.rollupAddress;
  config.contractDeploymentEmitterContract = deployL1ContractsValues.contractDeploymentEmitterAddress;
  config.inboxContract = deployL1ContractsValues.inboxAddress;

  const aztecNode = await AztecNodeService.createAndSync(config);
  const aztecRpcServer = await createAztecRPCServer(aztecNode);
  const accountCollection = new AccountCollection();
  const wasm = await CircuitsWasm.get();

  for (let i = 0; i < numberOfAccounts; ++i) {
    // We use the well-known private key and the validating account contract for the first account,
    // and generate random key pairs for the rest.
    // TODO(#662): Let the aztec rpc server generate the key pair rather than hardcoding the private key
    const privateKey = i === 0 ? Buffer.from(privKey!) : randomBytes(32);
    const publicKey = await generatePublicKey(privateKey);
    const salt = Fr.random();
    const deploymentData = await getContractDeploymentInfo(SchnorrAccountContractAbi, [], salt, publicKey);
    await aztecRpcServer.addAccount(
      privateKey,
      deploymentData.address,
      deploymentData.partialAddress,
      SchnorrAccountContractAbi,
    );

    const contractDeployer = new ContractDeployer(SchnorrAccountContractAbi, aztecRpcServer, publicKey);
    const deployMethod = contractDeployer.deploy();
    const tx = deployMethod.send({ contractAddressSalt: salt });
    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();
    if (receipt.status !== TxStatus.MINED) {
      throw new Error(`Deployment tx not mined (status is ${receipt.status})`);
    }
    const receiptAddress = receipt.contractAddress!;
    if (!receiptAddress.equals(deploymentData.address)) {
      throw new Error(
        `Deployment address does not match for account contract (expected ${deploymentData.address} got ${receiptAddress})`,
      );
    }
    accountCollection.registerAccount(
      deploymentData.address,
      new AccountContract(
        deploymentData.address,
        publicKey,
        new SchnorrAuthProvider(await Schnorr.new(), privateKey),
        deploymentData.partialAddress,
        SchnorrAccountContractAbi,
        wasm,
      ),
    );
    logger(`Created account ${deploymentData.address.toString()} with public key ${publicKey.toString()}`);
  }

  const accounts = await aztecRpcServer.getAccounts();
  const wallet = new AccountWallet(aztecRpcServer, accountCollection);

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
 * Converts a point to a public key.
 * @param point - the point to convert to
 * @returns two big ints x,y representing the public key
 */
export function pointToPublicKey(point: Point) {
  const x = point.x.toBigInt();
  const y = point.y.toBigInt();
  return {
    x,
    y,
  };
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
  owner = { x: 0n, y: 0n },
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
  const deployer = new ContractDeployer(NonNativeTokenContractAbi, wallet);
  const tx = deployer.deploy(initialBalance, owner).send({
    portalContract: tokenPortalAddress,
    contractAddressSalt: Fr.random(),
  });
  await tx.isMined(0, 0.1);
  const receipt = await tx.getReceipt();
  if (receipt.status !== TxStatus.MINED) throw new Error(`Tx status is ${receipt.status}`);
  const l2Contract = new Contract(receipt.contractAddress!, NonNativeTokenContractAbi, wallet);
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
  const mappingStorageSlotSeparator = new Fr(BigInt(MAPPING_SLOT_PEDERSEN_SEPARATOR)); // The pedersen domain separator for storage slot calculations.

  // Based on `at` function in
  // aztec3-packages/yarn-project/noir-contracts/src/contracts/noir-aztec/src/state_vars/map.nr
  const storageSlot = Fr.fromBuffer(
    pedersenPlookupCommitInputs(
      wasm,
      [mappingStorageSlotSeparator, mappingStorageSlot, key].map(f => f.toBuffer()),
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
  logger: Logger,
  aztecNode: AztecNodeService,
  contract: Contract,
  slot: bigint,
  key: Fr,
  expectedValue: bigint,
) {
  const storageSlot = await calculateAztecStorageSlot(slot, key);
  const storageValue = await aztecNode.getStorageAt(contract.address!, storageSlot.value);
  if (storageValue === undefined) {
    throw new Error(`Storage slot ${storageSlot} not found`);
  }

  const balance = toBigIntBE(storageValue);

  logger(`Account ${key.toShortString()} balance: ${balance}`);
  expect(balance).toBe(expectedValue);
}
