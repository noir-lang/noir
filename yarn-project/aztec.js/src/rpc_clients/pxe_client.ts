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
export const createPXEClient = (url: string, fetch = makeFetch([1, 2, 3], false)): PXE =>
  createJsonRpcClient<PXE>(
    url,
    {
      AuthWitness,
      AztecAddress,
      CompleteAddress,
      ContractData,
      FunctionSelector,
      EthAddress,
      ExtendedContractData,
      ExtendedNote,
      ExtendedUnencryptedL2Log,
      Fr,
      GrumpkinScalar,
      L2Block,
      L2Tx,
      LogId,
      Note,
      Point,
      TxExecutionRequest,
      TxHash,
    },
    { Tx, TxReceipt, L2BlockL2Logs },
    false,
    'pxe',
    fetch,
  );
