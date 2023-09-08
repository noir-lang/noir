import { getHttpRpcServer } from '@aztec/aztec-rpc';
import { DeployL1Contracts } from '@aztec/ethereum';
import { AztecRPC } from '@aztec/types';

import http from 'http';

import { createApiRouter } from './routes.js';

/**
 * Creates an http server that forwards calls to the rpc server and starts it on the given port.
 * @param aztecRpcServer - RPC server that answers queries to the created HTTP server.
 * @param deployedL1Contracts - Info on L1 deployed contracts.
 * @param port - Port to listen in.
 * @returns A running http server.
 */
export function startHttpRpcServer(
  aztecRpcServer: AztecRPC,
  deployedL1Contracts: DeployL1Contracts,
  port: string | number,
): http.Server {
  const rpcServer = getHttpRpcServer(aztecRpcServer);

  const app = rpcServer.getApp();
  const apiRouter = createApiRouter(deployedL1Contracts);
  app.use(apiRouter.routes());
  app.use(apiRouter.allowedMethods());

  const httpServer = http.createServer(app.callback());
  httpServer.listen(port);

  return httpServer;
}
