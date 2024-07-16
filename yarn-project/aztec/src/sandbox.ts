#!/usr/bin/env -S node --no-warnings
import { type AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { AztecAddress, SignerlessWallet, type Wallet } from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { type AztecNode } from '@aztec/circuit-types';
import { CANONICAL_AUTH_REGISTRY_ADDRESS, CANONICAL_KEY_REGISTRY_ADDRESS } from '@aztec/circuits.js';
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
import { AuthRegistryContract, KeyRegistryContract } from '@aztec/noir-contracts.js';
import { GasTokenContract } from '@aztec/noir-contracts.js/GasToken';
import { getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';
import { getCanonicalAuthRegistry } from '@aztec/protocol-contracts/auth-registry';
import { GasTokenAddress, getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalKeyRegistry } from '@aztec/protocol-contracts/key-registry';
import { type PXEServiceConfig, createPXEService, getPXEServiceConfig } from '@aztec/pxe';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { type HDAccount, type PrivateKeyAccount, createPublicClient, http as httpViemTransport } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

export const { MNEMONIC = 'test test test test test test test test test test test junk' } = process.env;

const logger = createDebugLogger('aztec:sandbox');

const localAnvil = foundry;

/**
 * Helper function that waits for the Ethereum RPC server to respond before deploying L1 contracts.
 */
async function waitThenDeploy(config: AztecNodeConfig, deployFunction: () => Promise<DeployL1Contracts>) {
  const chain = createEthereumChain(config.rpcUrl, config.l1ChainId);
  // wait for ETH RPC to respond to a request.
  const publicClient = createPublicClient({
    chain: chain.chainInfo,
    transport: httpViemTransport(chain.rpcUrl),
  });
  const l1ChainID = await retryUntil(
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

  if (!l1ChainID) {
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
    deployL1Contracts(aztecNodeConfig.rpcUrl, hdAccount, localAnvil, contractDeployLogger, l1Artifacts, {
      l2GasTokenAddress: GasTokenAddress,
      vkTreeRoot: getVKTreeRoot(),
    }),
  );

  aztecNodeConfig.l1Contracts = l1Contracts.l1ContractAddresses;

  return aztecNodeConfig.l1Contracts;
}

/**
 * Deploys the contract to pay for gas on L2.
 */
async function deployCanonicalL2GasToken(deployer: Wallet, l1ContractAddresses: L1ContractAddresses) {
  const gasPortalAddress = l1ContractAddresses.gasPortalAddress;
  const canonicalGasToken = getCanonicalGasToken();

  if (await deployer.isContractClassPubliclyRegistered(canonicalGasToken.contractClass.id)) {
    return;
  }

  const gasToken = await GasTokenContract.deploy(deployer)
    .send({ universalDeploy: true, contractAddressSalt: canonicalGasToken.instance.salt })
    .deployed();
  await gasToken.methods.set_portal(gasPortalAddress).send().wait();

  if (!gasToken.address.equals(canonicalGasToken.address)) {
    throw new Error(
      `Deployed Gas Token address ${gasToken.address} does not match expected address ${canonicalGasToken.address}`,
    );
  }

  if (!(await deployer.isContractPubliclyDeployed(canonicalGasToken.address))) {
    throw new Error(`Failed to deploy Gas Token to ${canonicalGasToken.address}`);
  }

  logger.info(`Deployed Gas Token on L2 at ${canonicalGasToken.address}`);
}

/**
 * Deploys the key registry on L2.
 */
async function deployCanonicalKeyRegistry(deployer: Wallet) {
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

  logger.info(`Deployed Key Registry on L2 at ${canonicalKeyRegistry.address}`);
}

/**
 * Deploys the auth registry on L2.
 */
async function deployCanonicalAuthRegistry(deployer: Wallet) {
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

  logger.info(`Deployed Auth Registry on L2 at ${canonicalAuthRegistry.address}`);
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

  const node = await createAztecNode(new NoopTelemetryClient(), aztecNodeConfig);
  const pxe = await createAztecPXE(node);

  await deployCanonicalKeyRegistry(
    new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(aztecNodeConfig.l1ChainId, aztecNodeConfig.version)),
  );
  await deployCanonicalAuthRegistry(
    new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(aztecNodeConfig.l1ChainId, aztecNodeConfig.version)),
  );

  if (config.enableGas) {
    await deployCanonicalL2GasToken(
      new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(aztecNodeConfig.l1ChainId, aztecNodeConfig.version)),
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
export async function createAztecNode(telemetryClient: TelemetryClient, config: Partial<AztecNodeConfig> = {}) {
  const aztecNodeConfig: AztecNodeConfig = { ...getConfigEnvVars(), ...config };
  const node = await AztecNodeService.createAndSync(aztecNodeConfig, telemetryClient);
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
