#!/usr/bin/env -S node --no-warnings
import { createDebugLogger } from '@aztec/foundation/log';

import http from 'http';

import { type AztecNodeConfig, AztecNodeService, createAztecNodeRpcServer, getConfigEnvVars } from '../index.js';

const { AZTEC_NODE_PORT = 8081, API_PREFIX = '' } = process.env;

const logger = createDebugLogger('aztec:node');

/**
 * Creates the node from provided config
 */
async function createAndDeployAztecNode() {
  const aztecNodeConfig: AztecNodeConfig = { ...getConfigEnvVars() };

  return await AztecNodeService.createAndSync(aztecNodeConfig);
}

/**
 * Create and start a new Aztec Node HTTP Server
 */
async function main() {
  logger.info(`Setting up Aztec Node...`);

  const aztecNode = await createAndDeployAztecNode();

  const shutdown = async () => {
    logger.info('Shutting down...');
    await aztecNode.stop();
    process.exit(0);
  };

  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);

  const rpcServer = createAztecNodeRpcServer(aztecNode);
  const app = rpcServer.getApp(API_PREFIX);

  const httpServer = http.createServer(app.callback());
  httpServer.listen(+AZTEC_NODE_PORT);
  logger.info(`Aztec Node JSON-RPC Server listening on port ${AZTEC_NODE_PORT}`);
}

main().catch(err => {
  logger.error(err);
  process.exit(1);
});
