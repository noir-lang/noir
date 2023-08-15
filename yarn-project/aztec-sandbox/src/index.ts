import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { createAztecRPCServer, getHttpRpcServer, getConfigEnvVars as getRpcConfigEnvVars } from '@aztec/aztec-rpc';
import { deployInitialSandboxAccounts } from '@aztec/aztec.js';
import { PrivateKey } from '@aztec/circuits.js';
import { deployL1Contracts } from '@aztec/ethereum';
import { createDebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';

import http from 'http';
import { HDAccount, createPublicClient, http as httpViemTransport } from 'viem';
import { mnemonicToAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

import { createApiRouter } from './routes.js';
import { github, splash } from './splash.js';

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
    600,
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
  const rpcConfig = getRpcConfigEnvVars();
  const hdAccount = mnemonicToAccount(MNEMONIC);
  const privKey = hdAccount.getHdKey().privateKey;

  const deployedL1Contracts = await waitThenDeploy(aztecNodeConfig.rpcUrl, hdAccount);
  aztecNodeConfig.publisherPrivateKey = new PrivateKey(Buffer.from(privKey!));
  aztecNodeConfig.rollupContract = deployedL1Contracts.rollupAddress;
  aztecNodeConfig.contractDeploymentEmitterContract = deployedL1Contracts.contractDeploymentEmitterAddress;
  aztecNodeConfig.inboxContract = deployedL1Contracts.inboxAddress;

  const aztecNode = await AztecNodeService.createAndSync(aztecNodeConfig);
  const aztecRpcServer = await createAztecRPCServer(aztecNode, rpcConfig);

  logger('Deploying initial accounts...');
  const accounts = await deployInitialSandboxAccounts(aztecRpcServer);

  const shutdown = async () => {
    logger('Shutting down...');
    await aztecRpcServer.stop();
    await aztecNode.stop();
    process.exit(0);
  };

  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);

  const rpcServer = getHttpRpcServer(aztecRpcServer);

  const app = rpcServer.getApp();
  const apiRouter = createApiRouter(deployedL1Contracts);
  app.use(apiRouter.routes());
  app.use(apiRouter.allowedMethods());

  const httpServer = http.createServer(app.callback());
  httpServer.listen(SERVER_PORT);
  logger.info(`Aztec JSON RPC listening on port ${SERVER_PORT}`);
  const accountStrings = [`Initial Accounts:\n\n`];

  const registeredAccounts = await aztecRpcServer.getAccounts();
  for (const account of accounts) {
    const completedAddress = await account.account.getCompleteAddress();
    if (registeredAccounts.find(a => a.equals(completedAddress.address))) {
      accountStrings.push(` Address: ${completedAddress.address.toString()}\n`);
      accountStrings.push(` Partial Address: ${completedAddress.partialAddress.toString()}\n`);
      accountStrings.push(` Private Key: ${account.privateKey.toString()}\n`);
      accountStrings.push(` Public Key: ${completedAddress.publicKey.toString()}\n\n`);
    }
  }
  logger.info(`${splash}\n${github}\n\n`.concat(...accountStrings));
}

main().catch(err => {
  logger.fatal(err);
  process.exit(1);
});
