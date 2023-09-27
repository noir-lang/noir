import { AztecAddress, CompleteAddress, EthAddress, Fr, GrumpkinScalar, Point } from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';
import {
  AuthWitness,
  ContractData,
  ExtendedContractData,
  L2BlockL2Logs,
  L2Tx,
  NotePreimage,
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
      L2Tx,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    fetch,
  );
