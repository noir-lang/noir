import {
  ContractData,
  EncodedContractFunction,
  ExtendedContractData,
  ExtendedUnencryptedL2Log,
  L1ToL2Message,
  L2Block,
  L2BlockL2Logs,
} from '@aztec/circuit-types';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { Archiver } from '../index.js';

/**
 * Wrap an Archiver instance with a JSON RPC HTTP server.
 * @param archiverService - The Archiver instance
 * @returns An JSON-RPC HTTP server
 */
export function createArchiverRpcServer(archiverService: Archiver): JsonRpcServer {
  return new JsonRpcServer(
    archiverService,
    {
      ContractData,
      EncodedContractFunction,
      EthAddress,
      ExtendedContractData,
      ExtendedUnencryptedL2Log,
      Fr,
      L1ToL2Message,
      L2Block,
      L2BlockL2Logs,
    },
    {},
    ['start', 'stop'],
  );
}
