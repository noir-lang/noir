import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc';
import {
  ContractData,
  ContractDeploymentTx,
  ContractPublicData,
  PrivateKey,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

import { foundry } from 'viem/chains';

import { AztecRPCServer, EthAddress } from '../index.js';

export const localAnvil = foundry;

/**
 * Wraps an instance of the Aztec RPC Server implementation to a JSON RPC HTTP interface.
 * @returns A new instance of the HTTP server.
 */
export function getHttpRpcServer(aztecRpcServer: AztecRPCServer): JsonRpcServer {
  const generatedRpcServer = new JsonRpcServer(
    aztecRpcServer,
    {
      AztecAddress,
      TxExecutionRequest,
      ContractData,
      ContractPublicData,
      TxHash,
      EthAddress,
      Point,
      PrivateKey,
      Fr,
    },
    { Tx, ContractDeploymentTx, TxReceipt },
    false,
    ['start', 'stop'],
  );
  return generatedRpcServer;
}
