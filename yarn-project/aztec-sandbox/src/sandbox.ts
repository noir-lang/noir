#!/usr/bin/env -S node --no-warnings
import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import {
  DeployL1Contracts,
  L1ContractArtifactsForDeployment,
  createEthereumChain,
  deployL1Contracts,
} from '@aztec/ethereum';
import { createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import {
  ContractDeploymentEmitterAbi,
  ContractDeploymentEmitterBytecode,
  InboxAbi,
  InboxBytecode,
  OutboxAbi,
  OutboxBytecode,
  RegistryAbi,
  RegistryBytecode,
  RollupAbi,
  RollupBytecode,
} from '@aztec/l1-artifacts';
import { createAztecRPCServer, getConfigEnvVars as getRpcConfigEnvVars } from '@aztec/pxe';

import { createPublicClient, http as httpViemTransport } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

const { MNEMONIC = 'test test test test test test test test test test test junk' } = process.env;

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

/** Sandbox settings. */
export type SandboxConfig = AztecNodeConfig & {
  /** Mnemonic used to derive the L1 deployer private key.*/
  l1Mnemonic: string;
};

/**
 * Create and start a new Aztec Node and RPC Server. Deploys L1 contracts.
 * Does not start any HTTP services nor populate any initial accounts.
 * @param config - Optional Sandbox settings.
 */
export async function createSandbox(config: Partial<SandboxConfig> = {}) {
  const aztecNodeConfig: AztecNodeConfig = { ...getConfigEnvVars(), ...config };
  const rpcConfig = getRpcConfigEnvVars();
  const hdAccount = mnemonicToAccount(config.l1Mnemonic ?? MNEMONIC);
  const privKey = hdAccount.getHdKey().privateKey;

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

  const l1Contracts = await waitThenDeploy(aztecNodeConfig, () =>
    deployL1Contracts(aztecNodeConfig.rpcUrl, hdAccount, localAnvil, logger, l1Artifacts),
  );
  aztecNodeConfig.publisherPrivateKey = `0x${Buffer.from(privKey!).toString('hex')}`;
  aztecNodeConfig.l1Contracts.rollupAddress = l1Contracts.l1ContractAddresses.rollupAddress;
  aztecNodeConfig.l1Contracts.contractDeploymentEmitterAddress =
    l1Contracts.l1ContractAddresses.contractDeploymentEmitterAddress;
  aztecNodeConfig.l1Contracts.inboxAddress = l1Contracts.l1ContractAddresses.inboxAddress;
  aztecNodeConfig.l1Contracts.registryAddress = l1Contracts.l1ContractAddresses.registryAddress;

  const node = await AztecNodeService.createAndSync(aztecNodeConfig);
  const rpcServer = await createAztecRPCServer(node, rpcConfig);

  const stop = async () => {
    await rpcServer.stop();
    await node.stop();
  };

  return { node, rpcServer, l1Contracts, stop };
}
