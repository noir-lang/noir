import { TxHash } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { type BotRunner } from './runner.js';

/**
 * Wraps a bot runner with a JSON RPC HTTP server.
 * @param botRunner - The BotRunner.
 * @returns An JSON-RPC HTTP server
 */
export function createBotRunnerRpcServer(botRunner: BotRunner) {
  return new JsonRpcServer(botRunner, { AztecAddress, EthAddress, Fr, TxHash }, {}, []);
}
