import { AztecAddress, EthAddress, Fr, Point } from '@aztec/circuits.js';
import { createJsonRpcClient, defaultFetch } from '@aztec/foundation/json-rpc';
import {
  AztecRPC,
  ContractData,
  ContractDeploymentTx,
  ContractPublicData,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

export const createAztecRpcClient = (url: string, fetch = defaultFetch): AztecRPC =>
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
    { Tx, ContractDeploymentTx, TxReceipt },
    false,
    fetch,
  );
