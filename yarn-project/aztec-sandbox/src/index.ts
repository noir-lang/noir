import http from 'http';
import { foundry } from 'viem/chains';
import { http as httpViemTransport, createPublicClient, HDAccount } from 'viem';

import { mnemonicToAccount } from 'viem/accounts';
import { getHttpRpcServer } from '@aztec/aztec-rpc';
import { createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import { AztecNodeConfig, getConfigEnvVars } from '@aztec/aztec-node';
import { deployL1Contracts } from '@aztec/ethereum';

import { createApiRouter } from './routes.js';

const { SERVER_PORT = 8080, MNEMONIC = 'test test test test test test test test test test test junk' } = process.env;

const logger = createDebugLogger('aztec:sandbox');

export const localAnvil = foundry;

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
        logger(`Failed to get Chain ID. Retrying...`);
      }
      return chainId;
    },
    'isEthRpcReady',
    30,
    1,
  );

  if (!chainID) {
    throw Error(`ETH RPC server unresponsive at ${rpcUrl}.`);
  }

  // Deploy L1 contracts
  const deployedL1Contracts = await deployL1Contracts(rpcUrl, hdAccount, localAnvil, logger);
  return deployedL1Contracts;
}

/**
 * Create and start a new Aztec RCP HTTP Server
 */
async function main() {
  const aztecNodeConfig: AztecNodeConfig = getConfigEnvVars();
  const hdAccount = mnemonicToAccount(MNEMONIC);
  const privKey = hdAccount.getHdKey().privateKey;

  const deployedL1Contracts = await waitThenDeploy(aztecNodeConfig.rpcUrl, hdAccount);
  aztecNodeConfig.publisherPrivateKey = Buffer.from(privKey!);
  aztecNodeConfig.rollupContract = deployedL1Contracts.rollupAddress;
  aztecNodeConfig.contractDeploymentEmitterContract = deployedL1Contracts.contractDeploymentEmitterAddress;
  aztecNodeConfig.inboxContract = deployedL1Contracts.inboxAddress;

  const rpcServer = await getHttpRpcServer(aztecNodeConfig);

  const app = rpcServer.getApp();
  const apiRouter = createApiRouter(deployedL1Contracts);
  app.use(apiRouter.routes());
  app.use(apiRouter.allowedMethods());

  const httpServer = http.createServer(app.callback());
  httpServer.listen(SERVER_PORT);
}

main()
  .then(() => logger(`Aztec JSON RPC listening on port ${SERVER_PORT}`))
  .catch(err => {
    logger(err);
    process.exit(1);
  });
