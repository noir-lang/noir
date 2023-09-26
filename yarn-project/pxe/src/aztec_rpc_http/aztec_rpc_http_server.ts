import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, GrumpkinScalar, Point } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';
import {
  AuthWitness,
  AztecRPC,
  CompleteAddress,
  ContractData,
  ExtendedContractData,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  NotePreimage,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

import http from 'http';
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
      AuthWitness,
      L2Block,
      L2Tx,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    ['start', 'stop'],
  );
  return generatedRpcServer;
}

/**
 * Creates an http server that forwards calls to the rpc server and starts it on the given port.
 * @param aztecRpcServer - RPC server that answers queries to the created HTTP server.
 * @param port - Port to listen in.
 * @returns A running http server.
 */
export function startHttpRpcServer(aztecRpcServer: AztecRPC, port: string | number): http.Server {
  const rpcServer = getHttpRpcServer(aztecRpcServer);

  const app = rpcServer.getApp();
  const httpServer = http.createServer(app.callback());
  httpServer.listen(port);

  return httpServer;
}
