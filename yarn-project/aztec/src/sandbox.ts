#!/usr/bin/env -S node --no-warnings
import { type AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { type AztecAddress, BatchCall, SignerlessWallet, type Wallet } from '@aztec/aztec.js';
import { deployInstance, registerContractClass } from '@aztec/aztec.js/deployment';
import { type AztecNode } from '@aztec/circuit-types';
import { DefaultMultiCallEntrypoint } from '@aztec/entrypoints/multi-call';
import {
  type DeployL1Contracts,
  type L1ContractAddresses,
  type L1ContractArtifactsForDeployment,
  NULL_KEY,
  createEthereumChain,
  deployL1Contracts,
} from '@aztec/ethereum';
import { createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
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
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { type PXEServiceConfig, createPXEService, getPXEServiceConfig } from '@aztec/pxe';

import {
  type HDAccount,
  type PrivateKeyAccount,
  createPublicClient,
  getContract,
  http as httpViemTransport,
} from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

export const { MNEMONIC = 'test test test test test test test test test test test junk' } = process.env;

const logger = createDebugLogger('aztec:sandbox');

const localAnvil = foundry;

/**
 * Helper function that waits for the Ethereum RPC server to respond before deploying L1 contracts.
 */
async function waitThenDeploy(config: AztecNodeConfig, deployFunction: () => Promise<DeployL1Contracts>) {
  const chain = createEthereumChain(config.rpcUrl, config.apiKey);
  // wait for ETH RPC to respond to a request.
  const publicClient = createPublicClient({
    chain: chain.chainInfo,
    transport: httpViemTransport(chain.rpcUrl),
  });
  const chainID = await retryUntil(
    async () => {
      let chainId = 0;
      try {
        chainId = await publicClient.getChainId();
      } catch (err) {
        logger.warn(`Failed to connect to Ethereum node at ${chain.rpcUrl}. Retrying...`);
      }
      return chainId;
    },
    'isEthRpcReady',
    600,
    1,
  );

  if (!chainID) {
    throw Error(`Ethereum node unresponsive at ${chain.rpcUrl}.`);
  }

  // Deploy L1 contracts
  return await deployFunction();
}

/**
 * Function to deploy our L1 contracts to the sandbox L1
 * @param aztecNodeConfig - The Aztec Node Config
 * @param hdAccount - Account for publishing L1 contracts
 */
export async function deployContractsToL1(
  aztecNodeConfig: AztecNodeConfig,
  hdAccount: HDAccount | PrivateKeyAccount,
  contractDeployLogger = logger,
) {
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

  const l1Contracts = await waitThenDeploy(aztecNodeConfig, () =>
    deployL1Contracts(aztecNodeConfig.rpcUrl, hdAccount, localAnvil, contractDeployLogger, l1Artifacts),
  );
  await initL1GasPortal(l1Contracts, getCanonicalGasToken(l1Contracts.l1ContractAddresses.gasPortalAddress).address);

  aztecNodeConfig.l1Contracts = l1Contracts.l1ContractAddresses;

  return aztecNodeConfig.l1Contracts;
}

/**
 * Initializes the portal between L1 and L2 used to pay for gas.
 * @param l1Data - The deployed L1 data.
 */
async function initL1GasPortal(
  { walletClient, l1ContractAddresses }: DeployL1Contracts,
  l2GasTokenAddress: AztecAddress,
) {
  const gasPortal = getContract({
    address: l1ContractAddresses.gasPortalAddress.toString(),
    abi: GasPortalAbi,
    client: walletClient,
  });

  await gasPortal.write.initialize(
    [
      l1ContractAddresses.registryAddress.toString(),
      l1ContractAddresses.gasTokenAddress.toString(),
      l2GasTokenAddress.toString(),
    ],
    {} as any,
  );

  logger(
    `Initialized Gas Portal at ${l1ContractAddresses.gasPortalAddress} to bridge between L1 ${l1ContractAddresses.gasTokenAddress} to L2 ${l2GasTokenAddress}`,
  );
}

/**
 * Deploys the contract to pay for gas on L2.
 */
async function deployCanonicalL2GasToken(deployer: Wallet, l1ContractAddresses: L1ContractAddresses) {
  const gasPortalAddress = l1ContractAddresses.gasPortalAddress;
  const canonicalGasToken = getCanonicalGasToken(gasPortalAddress);

  if (await deployer.isContractClassPubliclyRegistered(canonicalGasToken.contractClass.id)) {
    return;
  }

  const batch = new BatchCall(deployer, [
    (await registerContractClass(deployer, canonicalGasToken.artifact)).request(),
    deployInstance(deployer, canonicalGasToken.instance).request(),
  ]);

  await batch.send().wait();

  logger(`Deployed Gas Token on L2 at ${canonicalGasToken.address}`);
}

/** Sandbox settings. */
export type SandboxConfig = AztecNodeConfig & {
  /** Mnemonic used to derive the L1 deployer private key.*/
  l1Mnemonic: string;
  /** Enable the contracts to track and pay for gas */
  enableGas: boolean;
};

/**
 * Create and start a new Aztec Node and PXE. Deploys L1 contracts.
 * Does not start any HTTP services nor populate any initial accounts.
 * @param config - Optional Sandbox settings.
 */
export async function createSandbox(config: Partial<SandboxConfig> = {}) {
  const aztecNodeConfig: AztecNodeConfig = { ...getConfigEnvVars(), ...config };
  const hdAccount = mnemonicToAccount(config.l1Mnemonic ?? MNEMONIC);
  if (aztecNodeConfig.publisherPrivateKey === NULL_KEY) {
    const privKey = hdAccount.getHdKey().privateKey;
    aztecNodeConfig.publisherPrivateKey = `0x${Buffer.from(privKey!).toString('hex')}`;
  }

  if (!aztecNodeConfig.p2pEnabled) {
    await deployContractsToL1(aztecNodeConfig, hdAccount);
  }

  const node = await createAztecNode(aztecNodeConfig);
  const pxe = await createAztecPXE(node);

  if (config.enableGas) {
    await deployCanonicalL2GasToken(
      new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint()),
      aztecNodeConfig.l1Contracts,
    );
  }

  const stop = async () => {
    await pxe.stop();
    await node.stop();
  };

  return { node, pxe, aztecNodeConfig, stop };
}

/**
 * Create and start a new Aztec RPC HTTP Server
 * @param config - Optional Aztec node settings.
 */
export async function createAztecNode(config: Partial<AztecNodeConfig> = {}) {
  const aztecNodeConfig: AztecNodeConfig = { ...getConfigEnvVars(), ...config };
  const node = await AztecNodeService.createAndSync(aztecNodeConfig);
  return node;
}

/**
 * Create and start a new Aztec PXE HTTP Server
 * @param config - Optional PXE settings.
 */
export async function createAztecPXE(node: AztecNode, config: Partial<PXEServiceConfig> = {}) {
  const pxeServiceConfig: PXEServiceConfig = { ...getPXEServiceConfig(), ...config };
  const pxe = await createPXEService(node, pxeServiceConfig);
  return pxe;
}
