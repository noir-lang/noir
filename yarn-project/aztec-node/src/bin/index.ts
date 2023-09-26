#!/usr/bin/env -S node --no-warnings
import { createDebugLogger } from '@aztec/foundation/log';

import http from 'http';
import Koa from 'koa';
import Router from 'koa-router';

import { AztecNodeConfig, AztecNodeService, getConfigEnvVars, createAztecNodeRpcServer } from '../index.js';

const { SERVER_PORT = 8081, API_PREFIX = '' } = process.env;

const logger = createDebugLogger('aztec:node');

/**
 * Creates the node from provided config
 */
async function createAndDeployAztecNode() {
  const aztecNodeConfig: AztecNodeConfig = { ...getConfigEnvVars() };

  return await AztecNodeService.createAndSync(aztecNodeConfig);
}

/**
 * Creates a router for helper API endpoints of the Aztec RPC Server.
 * @param apiPrefix - The prefix to use for all api requests
 * @returns - The router for handling status requests.
 */
export function createStatusRouter(apiPrefix: string) {
  const router = new Router({ prefix: `${apiPrefix}` });
  router.get('/status', (ctx: Koa.Context) => {
    ctx.status = 200;
  });
  return router;
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
  const apiRouter = createStatusRouter(API_PREFIX);
  app.use(apiRouter.routes());
  app.use(apiRouter.allowedMethods());

  const httpServer = http.createServer(app.callback());
  httpServer.listen(+SERVER_PORT);
  logger.info(`Aztec Node JSON-RPC Server listening on port ${SERVER_PORT}`);
}

main().catch(err => {
  logger.error(err);
  process.exit(1);
});
