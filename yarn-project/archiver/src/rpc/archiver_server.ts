import {
  ExtendedUnencryptedL2Log,
  L2Block,
  L2BlockL2Logs,
  NullifierMembershipWitness,
  TxEffect,
  TxReceipt,
} from '@aztec/circuit-types';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { Archiver } from '../archiver/archiver.js';

/**
 * Wrap an Archiver instance with a JSON RPC HTTP server.
 * @param archiverService - The Archiver instance
 * @returns An JSON-RPC HTTP server
 */
export function createArchiverRpcServer(archiverService: Archiver): JsonRpcServer {
  return new JsonRpcServer(
    archiverService,
    {
      EthAddress,
      ExtendedUnencryptedL2Log,
      Fr,
      L2Block,
      L2BlockL2Logs,
      TxEffect,
    },
    { TxReceipt, NullifierMembershipWitness },
    ['start', 'stop'],
  );
}
