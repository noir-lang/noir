import {
  AztecAddress,
  CompleteAddress,
  EthAddress,
  Fr,
  FunctionSelector,
  GrumpkinScalar,
  Point,
} from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';
import {
  AuthWitness,
  ContractData,
  ExtendedContractData,
  ExtendedNote,
  ExtendedUnencryptedL2Log,
  L2BlockL2Logs,
  L2Tx,
  LogId,
  Note,
  PXE,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

export { makeFetch } from '@aztec/foundation/json-rpc/client';

export const createPXEClient = (url: string, fetch = makeFetch([1, 2, 3], true)): PXE =>
  createJsonRpcClient<PXE>(
    url,
    {
      CompleteAddress,
      FunctionSelector,
      AztecAddress,
      TxExecutionRequest,
      ContractData,
      ExtendedContractData,
      ExtendedUnencryptedL2Log,
      TxHash,
      EthAddress,
      Point,
      Fr,
      GrumpkinScalar,
      Note,
      ExtendedNote,
      AuthWitness,
      L2Tx,
      LogId,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    fetch,
  );
