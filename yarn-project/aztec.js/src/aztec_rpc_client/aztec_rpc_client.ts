import { AztecAddress, AztecRPC, EthAddress, Fr, Point, Tx } from '@aztec/aztec-rpc';
import { createJsonRpcClient } from '@aztec/foundation/json-rpc';
import { ContractData, ContractDeploymentTx, ContractPublicData, TxExecutionRequest, TxHash } from '@aztec/types';

export const createAztecRpcClient = (url: string): AztecRPC =>
  createJsonRpcClient<AztecRPC>(
    url,
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
  );
