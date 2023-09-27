import { createPXERpcServer } from '@aztec/pxe';
import { PXE } from '@aztec/types';

import http from 'http';

import { createApiRouter } from './routes.js';

/**
 * Creates an http server that forwards calls to the PXE and starts it on the given port.
 * @param pxe - PXE that answers queries to the created HTTP server.
 * @param port - Port to listen in.
 * @returns A running http server.
 */
export function startHttpRpcServer(pxe: PXE, port: string | number): http.Server {
  const rpcServer = createPXERpcServer(pxe);

  const app = rpcServer.getApp();
  const apiRouter = createApiRouter();
  app.use(apiRouter.routes());
  app.use(apiRouter.allowedMethods());

  const httpServer = http.createServer(app.callback());
  httpServer.listen(port);

  return httpServer;
}
