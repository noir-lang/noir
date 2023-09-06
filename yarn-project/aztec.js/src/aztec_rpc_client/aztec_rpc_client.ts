import { AztecAddress, CompleteAddress, EthAddress, Fr, Point, PrivateKey } from '@aztec/circuits.js';
import { createJsonRpcClient, defaultFetch } from '@aztec/foundation/json-rpc/client';
import {
  AztecRPC,
  ContractData,
  ExtendedContractData,
  L2BlockL2Logs,
  NotePreimage,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

export { makeFetch } from '@aztec/foundation/json-rpc/client';

export const createAztecRpcClient = (url: string, fetch = defaultFetch): AztecRPC =>
  createJsonRpcClient<AztecRPC>(
    url,
    {
      CompleteAddress,
      AztecAddress,
      TxExecutionRequest,
      ContractData,
      ExtendedContractData,
      TxHash,
      EthAddress,
      Point,
      PrivateKey,
      Fr,
      NotePreimage,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    fetch,
  );
