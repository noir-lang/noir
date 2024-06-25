#!/usr/bin/env -S node --no-warnings
import { Fr } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';
import { type Logger, createDebugLogger } from '@aztec/foundation/log';

import http from 'http';

import { TXEService } from '../txe_service/txe_service.js';
import { type ForeignCallResult, toForeignCallResult } from '../util/encoding.js';

const { TXE_PORT = 8080 } = process.env;

const logger = createDebugLogger('aztec:txe_service');

const TXESessions = new Map<number, TXEService>();

type MethodNames<T> = {
  [K in keyof T]: T[K] extends (...args: any[]) => any ? K : never;
}[keyof T];

type TXEForeignCallInput = {
  session_id: number;
  function: MethodNames<TXEService> | 'reset';
  inputs: any[];
};

class TXEDispatcher {
  constructor(private logger: Logger) {}

  // eslint-disable-next-line camelcase
  async resolve_foreign_call({
    session_id: sessionId,
    function: functionName,
    inputs,
  }: TXEForeignCallInput): Promise<ForeignCallResult> {
    this.logger.debug(`Calling ${functionName} on session ${sessionId}`);

    if (!TXESessions.has(sessionId) && functionName != 'reset') {
      this.logger.info(`Creating new session ${sessionId}`);
      TXESessions.set(sessionId, await TXEService.init(logger));
    }

    if (functionName === 'reset') {
      TXESessions.delete(sessionId) &&
        this.logger.info(`Called reset on session ${sessionId}, yeeting it out of existence`);
      return toForeignCallResult([]);
    } else {
      const txeService = TXESessions.get(sessionId);
      const response = await (txeService as any)[functionName](...inputs);
      return response;
    }
  }
}

/**
 * Creates an http server that forwards calls to the TXE and starts it on the given port.
 * @param txeService - TXE that answers queries to the created HTTP server.
 * @param port - Port to listen in.
 * @returns A running http server.
 */
export function startTXEHttpServer(dispatcher: TXEDispatcher, port: string | number): http.Server {
  const txeServer = new JsonRpcServer(dispatcher, { Fr }, {}, ['init']);

  const app = txeServer.getApp();
  const httpServer = http.createServer(app.callback());
  httpServer.timeout = 1e3 * 60 * 5; // 5 minutes
  httpServer.listen(port);

  return httpServer;
}

/**
 * Create and start a new TXE HTTP Server
 */
function main() {
  logger.info(`Setting up TXE...`);

  startTXEHttpServer(new TXEDispatcher(logger), TXE_PORT);

  logger.info(`TXE listening on port ${TXE_PORT}`);
}

main();
