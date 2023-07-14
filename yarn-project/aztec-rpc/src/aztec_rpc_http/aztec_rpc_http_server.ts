import { AztecNodeConfig, AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc';
import { ContractData, ContractDeploymentTx, ContractPublicData, Tx, TxExecutionRequest, TxHash } from '@aztec/types';
import { foundry } from 'viem/chains';

import { EthAddress, createAztecRPCServer, getConfigEnvVars } from '../index.js';

export const localAnvil = foundry;

/**
 * Wraps an instance of the Aztec RPC Server implementation to a JSON RPC HTTP interface.
 * @returns A new instance of the HTTP server.
 */
export async function getHttpRpcServer(nodeConfig: AztecNodeConfig): Promise<JsonRpcServer> {
  const aztecNode = await AztecNodeService.createAndSync(nodeConfig);
  const rpcServerConfig = getConfigEnvVars();
  const aztecRpcServer = await createAztecRPCServer(aztecNode, rpcServerConfig);
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
      Fr,
    },
    { Tx, ContractDeploymentTx },
    false,
    ['start', 'stop'],
  );
  return generatedRpcServer;
}
