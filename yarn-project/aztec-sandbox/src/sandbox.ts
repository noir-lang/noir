#!/usr/bin/env -S node --no-warnings
import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { createAztecRPCServer, getConfigEnvVars as getRpcConfigEnvVars } from '@aztec/aztec-rpc';
import { deployL1Contracts } from '@aztec/ethereum';
import { createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';

import { HDAccount, createPublicClient, http as httpViemTransport } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

const { MNEMONIC = 'test test test test test test test test test test test junk' } = process.env;

const logger = createDebugLogger('aztec:sandbox');

const localAnvil = foundry;

/**
 * Helper function that waits for the Ethereum RPC server to respond before deploying L1 contracts.
 */
async function waitThenDeploy(rpcUrl: string, hdAccount: HDAccount) {
  // wait for ETH RPC to respond to a request.
  const publicClient = createPublicClient({
    chain: foundry,
    transport: httpViemTransport(rpcUrl),
  });
  const chainID = await retryUntil(
    async () => {
      let chainId = 0;
      try {
        chainId = await publicClient.getChainId();
      } catch (err) {
        logger.warn(`Failed to connect to Ethereum node at ${rpcUrl}. Retrying...`);
      }
      return chainId;
    },
    'isEthRpcReady',
    600,
    1,
  );

  if (!chainID) {
    throw Error(`Ethereum node unresponsive at ${rpcUrl}.`);
  }

  // Deploy L1 contracts
  const deployedL1Contracts = await deployL1Contracts(rpcUrl, hdAccount, localAnvil, logger);
  return deployedL1Contracts;
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

  const l1Contracts = await waitThenDeploy(aztecNodeConfig.rpcUrl, hdAccount);
  aztecNodeConfig.publisherPrivateKey = `0x${Buffer.from(privKey!).toString('hex')}`;
  aztecNodeConfig.rollupContract = l1Contracts.rollupAddress;
  aztecNodeConfig.contractDeploymentEmitterContract = l1Contracts.contractDeploymentEmitterAddress;
  aztecNodeConfig.inboxContract = l1Contracts.inboxAddress;
  aztecNodeConfig.registryContract = l1Contracts.registryAddress;

  const node = await AztecNodeService.createAndSync(aztecNodeConfig);
  const rpcServer = await createAztecRPCServer(node, rpcConfig);

  const stop = async () => {
    await rpcServer.stop();
    await node.stop();
  };

  return { node, rpcServer, l1Contracts, stop };
}
