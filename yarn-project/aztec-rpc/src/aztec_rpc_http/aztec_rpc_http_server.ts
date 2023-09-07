import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, GrumpkinScalar, Point } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';
import {
  AztecRPC,
  CompleteAddress,
  ContractData,
  ExtendedContractData,
  L2BlockL2Logs,
  NotePreimage,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

import { foundry } from 'viem/chains';

import { EthAddress } from '../index.js';

export const localAnvil = foundry;

/**
 * Wraps an instance of the Aztec RPC Server implementation to a JSON RPC HTTP interface.
 * @returns A new instance of the HTTP server.
 */
export function getHttpRpcServer(aztecRpcServer: AztecRPC): JsonRpcServer {
  const generatedRpcServer = new JsonRpcServer(
    aztecRpcServer,
    {
      CompleteAddress,
      AztecAddress,
      TxExecutionRequest,
      ContractData,
      ExtendedContractData,
      TxHash,
      EthAddress,
      Point,
      Fr,
      GrumpkinScalar,
      NotePreimage,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    ['start', 'stop'],
  );
  return generatedRpcServer;
}
