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
  concatHex,
  createPublicClient,
  createWalletClient,
  encodeDeployData,
  getAddress,
  getContract,
  getContractAddress,
  http,
  numberToHex,
  padHex,
  zeroAddress,
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
   * Fee juice portal contract artifacts. Optional for now as gas is not strictly enforced
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
  args: {
    l2FeeJuiceAddress: AztecAddress;
    vkTreeRoot: Fr;
    assumeProvenUntil?: number;
    salt: number | undefined;
    initialValidators?: EthAddress[];
  },
): Promise<DeployL1Contracts> => {
  // We are assuming that you are running this on a local anvil node which have 1s block times
  // To align better with actual deployment, we update the block interval to 12s
  // The code is same as `setBlockInterval` in `cheat_codes.ts`
  const rpcCall = async (method: string, params: any[]) => {
    const paramsString = JSON.stringify(params);
    const content = {
      body: `{"jsonrpc":"2.0", "method": "${method}", "params": ${paramsString}, "id": 1}`,
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    };
    return await (await fetch(rpcUrl, content)).json();
  };
  if (chain.id == foundry.id) {
    const interval = 12;
    const res = await rpcCall('anvil_setBlockTimestampInterval', [interval]);
    if (res.error) {
      throw new Error(`Error setting block interval: ${res.error.message}`);
    }
    logger.info(`Set block interval to ${interval}`);
  }

  logger.info(`Deploying contracts from ${account.address.toString()}...`);

  const walletClient = createWalletClient({ account, chain, transport: http(rpcUrl) });
  const publicClient = createPublicClient({ chain, transport: http(rpcUrl) });
  const deployer = new L1Deployer(walletClient, publicClient, args.salt, logger);

  const registryAddress = await deployer.deploy(contractsToDeploy.registry, [account.address.toString()]);
  logger.info(`Deployed Registry at ${registryAddress}`);

  const availabilityOracleAddress = await deployer.deploy(contractsToDeploy.availabilityOracle);
  logger.info(`Deployed AvailabilityOracle at ${availabilityOracleAddress}`);

  const feeJuiceAddress = await deployer.deploy(contractsToDeploy.feeJuice);

  logger.info(`Deployed Fee Juice at ${feeJuiceAddress}`);

  const feeJuicePortalAddress = await deployer.deploy(contractsToDeploy.feeJuicePortal, [account.address.toString()]);

  logger.info(`Deployed Gas Portal at ${feeJuicePortalAddress}`);

  const rollupAddress = await deployer.deploy(contractsToDeploy.rollup, [
    getAddress(registryAddress.toString()),
    getAddress(availabilityOracleAddress.toString()),
    getAddress(feeJuicePortalAddress.toString()),
    args.vkTreeRoot.toString(),
    account.address.toString(),
    args.initialValidators?.map(v => v.toString()) ?? [],
  ]);
  logger.info(`Deployed Rollup at ${rollupAddress}`);

  await deployer.waitForDeployments();
  logger.info(`All contracts deployed`);

  const feeJuicePortal = getContract({
    address: feeJuicePortalAddress.toString(),
    abi: contractsToDeploy.feeJuicePortal.contractAbi,
    client: walletClient,
  });

  const feeJuice = getContract({
    address: feeJuiceAddress.toString(),
    abi: contractsToDeploy.feeJuice.contractAbi,
    client: walletClient,
  });

  const rollup = getContract({
    address: getAddress(rollupAddress.toString()),
    abi: contractsToDeploy.rollup.contractAbi,
    client: walletClient,
  });

  // Transaction hashes to await
  const txHashes: Hex[] = [];

  // @note  This value MUST match what is in `constants.nr`. It is currently specified here instead of just importing
  //        because there is circular dependency hell. This is a temporary solution. #3342
  // @todo  #8084
  // fund the portal contract with Fee Juice
  const FEE_JUICE_INITIAL_MINT = 20000000000;
  const mintTxHash = await feeJuice.write.mint([feeJuicePortalAddress.toString(), FEE_JUICE_INITIAL_MINT], {} as any);
  txHashes.push(mintTxHash);
  logger.info(`Funding fee juice portal contract with fee juice in ${mintTxHash}`);

  if ((await feeJuicePortal.read.registry([])) === zeroAddress) {
    const initPortalTxHash = await feeJuicePortal.write.initialize([
      registryAddress.toString(),
      feeJuiceAddress.toString(),
      args.l2FeeJuiceAddress.toString(),
    ]);
    txHashes.push(initPortalTxHash);
    logger.verbose(
      `Fee juice portal initializing with registry ${registryAddress.toString()} in tx ${initPortalTxHash}`,
    );
  } else {
    logger.verbose(`Fee juice portal is already initialized`);
  }

  logger.info(
    `Initialized Gas Portal at ${feeJuicePortalAddress} to bridge between L1 ${feeJuiceAddress} to L2 ${args.l2FeeJuiceAddress}`,
  );

  if (chain.id == foundry.id) {
    // @note  We make a time jump PAST the very first slot to not have to deal with the edge case of the first slot.
    //        The edge case being that the genesis block is already occupying slot 0, so we cannot have another block.
    try {
      // Need to get the time
      const currentSlot = (await rollup.read.getCurrentSlot([])) as bigint;

      if (BigInt(currentSlot) === 0n) {
        const ts = Number(await rollup.read.getTimestampForSlot([1]));
        await rpcCall('evm_setNextBlockTimestamp', [ts]);
        await rpcCall('hardhat_mine', [1]);
        const currentSlot = (await rollup.read.getCurrentSlot([])) as bigint;

        if (BigInt(currentSlot) !== 1n) {
          throw new Error(`Error jumping time: current slot is ${currentSlot}`);
        }
        logger.info(`Jumped to slot 1`);
      }
    } catch (e) {
      throw new Error(`Error jumping time: ${e}`);
    }
  }

  // Set initial blocks as proven if requested
  if (args.assumeProvenUntil && args.assumeProvenUntil > 0) {
    await rollup.write.setAssumeProvenUntilBlockNumber([BigInt(args.assumeProvenUntil)], { account });
    logger.info(`Set Rollup assumedProvenUntil to ${args.assumeProvenUntil}`);
  }

  // Inbox and Outbox are immutable and are deployed from Rollup's constructor so we just fetch them from the contract.
  const inboxAddress = EthAddress.fromString((await rollup.read.INBOX([])) as any);
  logger.info(`Inbox available at ${inboxAddress}`);

  const outboxAddress = EthAddress.fromString((await rollup.read.OUTBOX([])) as any);
  logger.info(`Outbox available at ${outboxAddress}`);

  // We need to call a function on the registry to set the various contract addresses.
  const registryContract = getContract({
    address: getAddress(registryAddress.toString()),
    abi: contractsToDeploy.registry.contractAbi,
    client: walletClient,
  });
  if (!(await registryContract.read.isRollupRegistered([getAddress(rollupAddress.toString())]))) {
    const upgradeTxHash = await registryContract.write.upgrade([getAddress(rollupAddress.toString())], { account });
    logger.verbose(
      `Upgrading registry contract at ${registryAddress} to rollup ${rollupAddress} in tx ${upgradeTxHash}`,
    );
    txHashes.push(upgradeTxHash);
  } else {
    logger.verbose(`Registry ${registryAddress} has already registered rollup ${rollupAddress}`);
  }

  // Wait for all actions to be mined
  await Promise.all(txHashes.map(txHash => publicClient.waitForTransactionReceipt({ hash: txHash })));
  logger.verbose(`All transactions for L1 deployment have been mined`);

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

class L1Deployer {
  private salt: Hex | undefined;
  private txHashes: Hex[] = [];
  constructor(
    private walletClient: WalletClient<HttpTransport, Chain, Account>,
    private publicClient: PublicClient<HttpTransport, Chain>,
    maybeSalt: number | undefined,
    private logger: DebugLogger,
  ) {
    this.salt = maybeSalt ? padHex(numberToHex(maybeSalt), { size: 32 }) : undefined;
  }

  async deploy(
    params: { contractAbi: Narrow<Abi | readonly unknown[]>; contractBytecode: Hex },
    args: readonly unknown[] = [],
  ): Promise<EthAddress> {
    const { txHash, address } = await deployL1Contract(
      this.walletClient,
      this.publicClient,
      params.contractAbi,
      params.contractBytecode,
      args,
      this.salt,
      this.logger,
    );
    if (txHash) {
      this.txHashes.push(txHash);
    }
    return address;
  }

  async waitForDeployments(): Promise<void> {
    await Promise.all(this.txHashes.map(txHash => this.publicClient.waitForTransactionReceipt({ hash: txHash })));
  }
}

// docs:start:deployL1Contract
/**
 * Helper function to deploy ETH contracts.
 * @param walletClient - A viem WalletClient.
 * @param publicClient - A viem PublicClient.
 * @param abi - The ETH contract's ABI (as abitype's Abi).
 * @param bytecode  - The ETH contract's bytecode.
 * @param args - Constructor arguments for the contract.
 * @param maybeSalt - Optional salt for CREATE2 deployment (does not wait for deployment tx to be mined if set, does not send tx if contract already exists).
 * @returns The ETH address the contract was deployed to.
 */
export async function deployL1Contract(
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  abi: Narrow<Abi | readonly unknown[]>,
  bytecode: Hex,
  args: readonly unknown[] = [],
  maybeSalt?: Hex,
  logger?: DebugLogger,
): Promise<{ address: EthAddress; txHash: Hex | undefined }> {
  let txHash: Hex | undefined = undefined;
  let address: Hex | null | undefined = undefined;

  if (maybeSalt) {
    const salt = padHex(maybeSalt, { size: 32 });
    const deployer: Hex = '0x4e59b44847b379578588920cA78FbF26c0B4956C';
    const calldata = encodeDeployData({ abi, bytecode, args });
    address = getContractAddress({ from: deployer, salt, bytecode: calldata, opcode: 'CREATE2' });
    const existing = await publicClient.getBytecode({ address });

    if (existing === undefined || existing === '0x') {
      txHash = await walletClient.sendTransaction({ to: deployer, data: concatHex([salt, calldata]) });
      logger?.verbose(`Deploying contract with salt ${salt} to address ${address} in tx ${txHash}`);
    } else {
      logger?.verbose(`Skipping existing deployment of contract with salt ${salt} to address ${address}`);
    }
  } else {
    txHash = await walletClient.deployContract({ abi, bytecode, args });
    logger?.verbose(`Deploying contract in tx ${txHash}`);
    const receipt = await publicClient.waitForTransactionReceipt({ hash: txHash, pollingInterval: 100 });
    address = receipt.contractAddress;
    if (!address) {
      throw new Error(
        `No contract address found in receipt: ${JSON.stringify(receipt, (_, val) =>
          typeof val === 'bigint' ? String(val) : val,
        )}`,
      );
    }
  }

  return { address: EthAddress.fromString(address!), txHash };
}
// docs:end:deployL1Contract
