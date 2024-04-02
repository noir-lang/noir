import {
  EncryptedL2BlockL2Logs,
  ExtendedUnencryptedL2Log,
  L2Block,
  NullifierMembershipWitness,
  TxEffect,
  TxReceipt,
  UnencryptedL2BlockL2Logs,
} from '@aztec/circuit-types';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { type Archiver } from '../archiver/archiver.js';

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
      EncryptedL2BlockL2Logs,
      UnencryptedL2BlockL2Logs,
      TxEffect,
    },
    { TxReceipt, NullifierMembershipWitness },
    ['start', 'stop'],
  );
}
