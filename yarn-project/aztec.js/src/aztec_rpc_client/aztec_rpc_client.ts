import { AztecAddress, CompleteAddress, EthAddress, Fr, Point, PrivateKey } from '@aztec/circuits.js';
import { createJsonRpcClient, defaultFetch } from '@aztec/foundation/json-rpc/client';
import {
  AztecRPC,
  ContractData,
  ContractDataAndBytecode,
  ContractDeploymentTx,
  L2BlockL2Logs,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

export { mustSucceedFetch } from '@aztec/foundation/json-rpc/client';
export { mustSucceedFetchUnlessNoRetry } from '@aztec/foundation/json-rpc/client';

export const createAztecRpcClient = (url: string, fetch = defaultFetch): AztecRPC =>
  createJsonRpcClient<AztecRPC>(
    url,
    {
      CompleteAddress,
      AztecAddress,
      TxExecutionRequest,
      ContractData,
      ContractDataAndBytecode,
      TxHash,
      EthAddress,
      Point,
      PrivateKey,
      Fr,
    },
    { Tx, ContractDeploymentTx, TxReceipt, L2BlockL2Logs },
    false,
    fetch,
  );
