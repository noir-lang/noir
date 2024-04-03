import {
  AuthWitness,
  EncryptedL2BlockL2Logs,
  ExtendedNote,
  ExtendedUnencryptedL2Log,
  L2Block,
  LogId,
  Note,
  NullifierMembershipWitness,
  type PXE,
  SimulatedTx,
  Tx,
  TxEffect,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
  UnencryptedL2BlockL2Logs,
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
      FunctionSelector,
      EthAddress,
      ExtendedNote,
      ExtendedUnencryptedL2Log,
      Fr,
      GrumpkinScalar,
      L2Block,
      TxEffect,
      LogId,
      Note,
      Point,
      TxExecutionRequest,
      TxHash,
    },
    { Tx, SimulatedTx, TxReceipt, EncryptedL2BlockL2Logs, UnencryptedL2BlockL2Logs, NullifierMembershipWitness },
    false,
    'pxe',
    fetch,
  );
