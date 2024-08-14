import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { type Fr } from '@aztec/foundation/fields';
import { type DebugLogger } from '@aztec/foundation/log';

import type { Abi, Narrow } from 'abitype';
import {
  type Account,
  type Chain,
  type Hex,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  createPublicClient,
  createWalletClient,
  getAddress,
  getContract,
  http,
} from 'viem';
import { type HDAccount, type PrivateKeyAccount, mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

import { type L1ContractAddresses } from './l1_contract_addresses.js';

/**
 * Return type of the deployL1Contract function.
 */
export type DeployL1Contracts = {
  /**
   * Wallet Client Type.
   */
  walletClient: WalletClient<HttpTransport, Chain, Account>;
  /**
   * Public Client Type.
   */
  publicClient: PublicClient<HttpTransport, Chain>;
  /**
   * The currently deployed l1 contract addresses
   */
  l1ContractAddresses: L1ContractAddresses;
};

/**
 * Contract artifacts
 */
export interface ContractArtifacts {
  /**
   * The contract abi.
   */
  contractAbi: Narrow<Abi | readonly unknown[]>;
  /**
   * The contract bytecode
   */
  contractBytecode: Hex;
}

/**
 * All L1 Contract Artifacts for deployment
 */
export interface L1ContractArtifactsForDeployment {
  /**
   * Inbox contract artifacts
   */
  inbox: ContractArtifacts;
  /**
   * Outbox contract artifacts
   */
  outbox: ContractArtifacts;
  /**
   * Availability Oracle contract artifacts
   */
  availabilityOracle: ContractArtifacts;
  /**
   * Registry contract artifacts
   */
  registry: ContractArtifacts;
  /**
   * Rollup contract artifacts
   */
  rollup: ContractArtifacts;
  /**
   * The token to pay for gas. This will be bridged to L2 via the feeJuicePortal below
   */
  feeJuice: ContractArtifacts;
  /**
   * Gas portal contract artifacts. Optional for now as gas is not strictly enforced
   */
  feeJuicePortal: ContractArtifacts;
}

export type L1Clients = {
  publicClient: PublicClient<HttpTransport, Chain>;
  walletClient: WalletClient<HttpTransport, Chain, Account>;
};

/**
 * Creates a wallet and a public viem client for interacting with L1.
 * @param rpcUrl - RPC URL to connect to L1.
 * @param mnemonicOrPrivateKeyOrHdAccount - Mnemonic or account for the wallet client.
 * @param chain - Optional chain spec (defaults to local foundry).
 * @returns - A wallet and a public client.
 */
export function createL1Clients(
  rpcUrl: string,
  mnemonicOrPrivateKeyOrHdAccount: string | `0x${string}` | HDAccount | PrivateKeyAccount,
  chain: Chain = foundry,
): L1Clients {
  const hdAccount =
    typeof mnemonicOrPrivateKeyOrHdAccount === 'string'
      ? mnemonicOrPrivateKeyOrHdAccount.startsWith('0x')
        ? privateKeyToAccount(mnemonicOrPrivateKeyOrHdAccount as `0x${string}`)
        : mnemonicToAccount(mnemonicOrPrivateKeyOrHdAccount)
      : mnemonicOrPrivateKeyOrHdAccount;

  const walletClient = createWalletClient({
    account: hdAccount,
    chain,
    transport: http(rpcUrl),
  });
  const publicClient = createPublicClient({
    chain,
    transport: http(rpcUrl),
  });

  return { walletClient, publicClient };
}

/**
 * Deploys the aztec L1 contracts; Rollup & (optionally) Decoder Helper.
 * @param rpcUrl - URL of the ETH RPC to use for deployment.
 * @param account - Private Key or HD Account that will deploy the contracts.
 * @param chain - The chain instance to deploy to.
 * @param logger - A logger object.
 * @param contractsToDeploy - The set of L1 artifacts to be deployed
 * @param args - Arguments for initialization of L1 contracts
 * @returns A list of ETH addresses of the deployed contracts.
 */
export const deployL1Contracts = async (
  rpcUrl: string,
  account: HDAccount | PrivateKeyAccount,
  chain: Chain,
  logger: DebugLogger,
  contractsToDeploy: L1ContractArtifactsForDeployment,
  args: { l2FeeJuiceAddress: AztecAddress; vkTreeRoot: Fr; assumeProvenUntil?: number },
): Promise<DeployL1Contracts> => {
  // We are assuming that you are running this on a local anvil node which have 1s block times
  // To align better with actual deployment, we update the block interval to 12s
  // The code is same as `setBlockInterval` in `cheat_codes.ts`
  const rpcCall = async (rpcUrl: string, method: string, params: any[]) => {
    const paramsString = JSON.stringify(params);
    const content = {
      body: `{"jsonrpc":"2.0", "method": "${method}", "params": ${paramsString}, "id": 1}`,
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    };
    return await (await fetch(rpcUrl, content)).json();
  };
  const interval = 12;
  const res = await rpcCall(rpcUrl, 'anvil_setBlockTimestampInterval', [interval]);
  if (res.error) {
    throw new Error(`Error setting block interval: ${res.error.message}`);
  }
  logger.info(`Set block interval to ${interval}`);

  logger.debug('Deploying contracts...');

  const walletClient = createWalletClient({
    account,
    chain,
    transport: http(rpcUrl),
  });
  const publicClient = createPublicClient({
    chain,
    transport: http(rpcUrl),
  });

  const registryAddress = await deployL1Contract(
    walletClient,
    publicClient,
    contractsToDeploy.registry.contractAbi,
    contractsToDeploy.registry.contractBytecode,
  );
  logger.info(`Deployed Registry at ${registryAddress}`);

  const availabilityOracleAddress = await deployL1Contract(
    walletClient,
    publicClient,
    contractsToDeploy.availabilityOracle.contractAbi,
    contractsToDeploy.availabilityOracle.contractBytecode,
  );
  logger.info(`Deployed AvailabilityOracle at ${availabilityOracleAddress}`);

  const feeJuiceAddress = await deployL1Contract(
    walletClient,
    publicClient,
    contractsToDeploy.feeJuice.contractAbi,
    contractsToDeploy.feeJuice.contractBytecode,
  );

  logger.info(`Deployed Fee Juice at ${feeJuiceAddress}`);

  const rollupAddress = await deployL1Contract(
    walletClient,
    publicClient,
    contractsToDeploy.rollup.contractAbi,
    contractsToDeploy.rollup.contractBytecode,
    [
      getAddress(registryAddress.toString()),
      getAddress(availabilityOracleAddress.toString()),
      getAddress(feeJuiceAddress.toString()),
      args.vkTreeRoot.toString(),
    ],
  );
  logger.info(`Deployed Rollup at ${rollupAddress}`);

  // Set initial blocks as proven if requested
  if (args.assumeProvenUntil && args.assumeProvenUntil > 0) {
    const rollup = getContract({
      address: getAddress(rollupAddress.toString()),
      abi: contractsToDeploy.rollup.contractAbi,
      client: walletClient,
    });
    await rollup.write.setAssumeProvenUntilBlockNumber([BigInt(args.assumeProvenUntil)], { account });
    logger.info(`Set Rollup assumedProvenUntil to ${args.assumeProvenUntil}`);
  }

  // Inbox and Outbox are immutable and are deployed from Rollup's constructor so we just fetch them from the contract.
  let inboxAddress!: EthAddress;
  {
    const rollup = getContract({
      address: getAddress(rollupAddress.toString()),
      abi: contractsToDeploy.rollup.contractAbi,
      client: publicClient,
    });
    inboxAddress = EthAddress.fromString((await rollup.read.INBOX([])) as any);
  }
  logger.info(`Inbox available at ${inboxAddress}`);

  let outboxAddress!: EthAddress;
  {
    const rollup = getContract({
      address: getAddress(rollupAddress.toString()),
      abi: contractsToDeploy.rollup.contractAbi,
      client: publicClient,
    });
    outboxAddress = EthAddress.fromString((await rollup.read.OUTBOX([])) as any);
  }
  logger.info(`Outbox available at ${outboxAddress}`);

  // We need to call a function on the registry to set the various contract addresses.
  const registryContract = getContract({
    address: getAddress(registryAddress.toString()),
    abi: contractsToDeploy.registry.contractAbi,
    client: walletClient,
  });
  await registryContract.write.upgrade([getAddress(rollupAddress.toString())], { account });

  // this contract remains uninitialized because at this point we don't know the address of the Fee Juice on L2
  const feeJuicePortalAddress = await deployL1Contract(
    walletClient,
    publicClient,
    contractsToDeploy.feeJuicePortal.contractAbi,
    contractsToDeploy.feeJuicePortal.contractBytecode,
  );

  logger.info(`Deployed Gas Portal at ${feeJuicePortalAddress}`);

  const feeJuicePortal = getContract({
    address: feeJuicePortalAddress.toString(),
    abi: contractsToDeploy.feeJuicePortal.contractAbi,
    client: walletClient,
  });

  await publicClient.waitForTransactionReceipt({
    hash: await feeJuicePortal.write.initialize([
      registryAddress.toString(),
      feeJuiceAddress.toString(),
      args.l2FeeJuiceAddress.toString(),
    ]),
  });

  logger.info(
    `Initialized Gas Portal at ${feeJuicePortalAddress} to bridge between L1 ${feeJuiceAddress} to L2 ${args.l2FeeJuiceAddress}`,
  );

  // fund the rollup contract with Fee Juice
  const feeJuice = getContract({
    address: feeJuiceAddress.toString(),
    abi: contractsToDeploy.feeJuice.contractAbi,
    client: walletClient,
  });
  const receipt = await feeJuice.write.mint([rollupAddress.toString(), 100000000000000000000n], {} as any);
  await publicClient.waitForTransactionReceipt({ hash: receipt });
  logger.info(`Funded rollup contract with Fee Juice`);

  const l1Contracts: L1ContractAddresses = {
    availabilityOracleAddress,
    rollupAddress,
    registryAddress,
    inboxAddress,
    outboxAddress,
    feeJuiceAddress,
    feeJuicePortalAddress,
  };

  return {
    walletClient,
    publicClient,
    l1ContractAddresses: l1Contracts,
  };
};

// docs:start:deployL1Contract
/**
 * Helper function to deploy ETH contracts.
 * @param walletClient - A viem WalletClient.
 * @param publicClient - A viem PublicClient.
 * @param abi - The ETH contract's ABI (as abitype's Abi).
 * @param bytecode  - The ETH contract's bytecode.
 * @param args - Constructor arguments for the contract.
 * @returns The ETH address the contract was deployed to.
 */
export async function deployL1Contract(
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  abi: Narrow<Abi | readonly unknown[]>,
  bytecode: Hex,
  args: readonly unknown[] = [],
): Promise<EthAddress> {
  const hash = await walletClient.deployContract({
    abi,
    bytecode,
    args,
  });

  const receipt = await publicClient.waitForTransactionReceipt({ hash, pollingInterval: 100 });
  const contractAddress = receipt.contractAddress;
  if (!contractAddress) {
    throw new Error(
      `No contract address found in receipt: ${JSON.stringify(receipt, (_, val) =>
        typeof val === 'bigint' ? String(val) : val,
      )}`,
    );
  }

  return EthAddress.fromString(receipt.contractAddress!);
}
// docs:end:deployL1Contract
