import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import {
  AztecAddress,
  AztecRPCServer,
  Contract,
  ContractDeployer,
  EthAddress,
  Point,
  SentTx,
  createAztecRPCServer,
} from '@aztec/aztec.js';
import { DeployL1Contracts, deployL1Contract, deployL1Contracts } from '@aztec/ethereum';
import { ContractAbi } from '@aztec/foundation/abi';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';
import { NonNativeTokenContractAbi } from '@aztec/noir-contracts/examples';
import every from 'lodash.every';
import zipWith from 'lodash.zipwith';
import { Account, Chain, HttpTransport, PublicClient, WalletClient, getContract } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { MNEMONIC, localAnvil, privateKey } from './fixtures.js';

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
  for (let i = 0; i < numberOfAccounts; ++i) {
    let address;
    if (i == 0) {
      // TODO(#662): Let the aztec rpc server generate the keypair rather than hardcoding the private key and generate all accounts as smart accounts
      const [txHash, newAddress] = await aztecRpcServer.createSmartAccount(privateKey);
      const isMined = await new SentTx(aztecRpcServer, Promise.resolve(txHash)).isMined();
      expect(isMined).toBeTruthy();
      address = newAddress;
    } else {
      address = await aztecRpcServer.addExternallyOwnedAccount();
    }
    const pubKey = await aztecRpcServer.getAccountPublicKey(address);
    logger(`Created account ${address.toString()} with public key ${pubKey.toString()}`);
  }

  const accounts = await aztecRpcServer.getAccounts();

  return {
    aztecNode,
    aztecRpcServer,
    deployL1ContractsValues,
    accounts,
    config,
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
 * @param aztecRpcServer - the RPC server to make the request.
 * @param abi - contracts to be deployed.
 * @returns The deployed contract instances.
 */
export async function deployL2Contracts(aztecRpcServer: AztecRPCServer, abis: ContractAbi[]) {
  const logger = getLogger();
  const calls = await Promise.all(abis.map(abi => new ContractDeployer(abi, aztecRpcServer).deploy()));
  for (const call of calls) await call.create();
  const txs = await Promise.all(calls.map(c => c.send()));
  expect(every(await Promise.all(txs.map(tx => tx.isMined(0, 0.1))))).toBeTruthy();
  const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
  const contracts = zipWith(
    abis,
    receipts,
    (abi, receipt) => new Contract(receipt!.contractAddress!, abi!, aztecRpcServer),
  );
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
  const x = point.buffer.subarray(0, 32);
  const y = point.buffer.subarray(32, 64);
  return {
    x: toBigIntBE(x),
    y: toBigIntBE(y),
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
 * @returns l2 contract instance, token portal instance, token portal address and the underlying ERC20 instance
 */
export async function deployAndInitializeNonNativeL2TokenContracts(
  aztecRpcServer: AztecRPCServer,
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  rollupRegistryAddress: EthAddress,
  initialBalance = 0n,
  owner = { x: 0n, y: 0n },
) {
  // deploy underlying contract
  const underlyingERC20Address = await deployL1Contract(
    walletClient,
    publicClient,
    PortalERC20Abi,
    PortalERC20Bytecode,
  );
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
  const deployer = new ContractDeployer(NonNativeTokenContractAbi, aztecRpcServer);
  const tx = deployer.deploy(initialBalance, owner).send({
    portalContract: tokenPortalAddress,
  });
  await tx.isMined(0, 0.1);
  const receipt = await tx.getReceipt();
  const l2Contract = new Contract(receipt.contractAddress!, NonNativeTokenContractAbi, aztecRpcServer);
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
