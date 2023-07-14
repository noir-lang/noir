import { AztecAddress, EthAddress, Fr, Point } from '@aztec/circuits.js';
import { createJsonRpcClient } from '@aztec/foundation/json-rpc';
import {
  AztecRPC,
  ContractData,
  ContractDeploymentTx,
  ContractPublicData,
  Tx,
  TxExecutionRequest,
  TxHash,
} from '@aztec/types';

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
