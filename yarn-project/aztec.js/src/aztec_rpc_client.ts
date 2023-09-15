import { AztecAddress, CompleteAddress, EthAddress, Fr, GrumpkinScalar, Point } from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';
import {
  AuthWitness,
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

export const createAztecRpcClient = (url: string, fetch = makeFetch([1, 2, 3], true)): AztecRPC =>
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
      Fr,
      GrumpkinScalar,
      NotePreimage,
      AuthWitness,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    fetch,
  );
