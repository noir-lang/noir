import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import http from 'http';

/**
 * Creates an http server that forwards calls to the underlying instance and starts it on the given port.
 * @param instance - Instance to wrap in a JSON-RPC server.
 * @param jsonRpcFactoryFunc - Function that wraps the instance in a JSON-RPC server.
 * @param port - Port to listen in.
 * @returns A running http server.
 */
export function startHttpRpcServer<T>(
  instance: T,
  jsonRpcFactoryFunc: (instance: T) => JsonRpcServer,
  port: string | number,
): http.Server {
  const rpcServer = jsonRpcFactoryFunc(instance);

  const app = rpcServer.getApp();

  const httpServer = http.createServer(app.callback());
  httpServer.listen(port);

  return httpServer;
}
