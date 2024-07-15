import { Fr } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';
import { type Logger } from '@aztec/foundation/log';

import { TXEService } from './txe_service/txe_service.js';
import { type ForeignCallResult, toForeignCallResult } from './util/encoding.js';

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
      TXESessions.set(sessionId, await TXEService.init(this.logger));
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
 * Creates an RPC server that forwards calls to the TXE.
 * @param logger - Logger to output to
 * @returns A TXE RPC server.
 */
export function createTXERpcServer(logger: Logger) {
  return new JsonRpcServer(new TXEDispatcher(logger), { Fr }, {}, ['init']);
}
