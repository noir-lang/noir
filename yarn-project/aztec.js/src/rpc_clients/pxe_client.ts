import {
  AuthWitness,
  ContractData,
  ExtendedContractData,
  ExtendedNote,
  ExtendedUnencryptedL2Log,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  LogId,
  Note,
  PXE,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/circuit-types';
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

/**
 * Creates a JSON-RPC client to remotely talk to PXE.
 * @param url - The URL of the PXE.
 * @param fetch - The fetch implementation to use.
 * @returns A JSON-RPC client of PXE.
 */
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
      L2Block,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    'pxe',
    fetch,
  );
