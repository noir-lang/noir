import {
  AztecAddress,
  CallRequest,
  Fr,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  Proof,
} from '@aztec/circuits.js';
import { makePrivateKernelTailCircuitPublicInputs, makePublicCallRequest } from '@aztec/circuits.js/testing';
import { ContractArtifact } from '@aztec/foundation/abi';
import { makeTuple } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { randomBytes } from '@aztec/foundation/crypto';
import { Tuple } from '@aztec/foundation/serialize';
import { ContractInstanceWithAddress, SerializableContractInstance } from '@aztec/types/contracts';

import { ExtendedContractData } from './contract_data.js';
import { DeployedContract } from './interfaces/index.js';
import { FunctionL2Logs, Note, TxL2Logs } from './logs/index.js';
import { ExtendedNote } from './notes/index.js';
import { Tx, TxHash } from './tx/index.js';

/**
 * Testing utility to create empty logs composed from a single empty log.
 */
export function makeEmptyLogs(): TxL2Logs {
  const functionLogs = [new FunctionL2Logs([Buffer.alloc(0)])];
  return new TxL2Logs(functionLogs);
}

export const randomTxHash = (): TxHash => new TxHash(randomBytes(32));

export const mockTx = (seed = 1) => {
  const tx = new Tx(
    makePrivateKernelTailCircuitPublicInputs(seed),
    new Proof(Buffer.alloc(0)),
    TxL2Logs.random(8, 3), // 8 priv function invocations creating 3 encrypted logs each
    TxL2Logs.random(11, 2), // 8 priv + 3 pub function invocations creating 2 unencrypted logs each
    times(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makePublicCallRequest),
    times(MAX_NEW_CONTRACTS_PER_TX, () => ExtendedContractData.random()) as Tuple<
      ExtendedContractData,
      typeof MAX_NEW_CONTRACTS_PER_TX
    >,
  );

  tx.data.endNonRevertibleData.publicCallStack = [
    tx.enqueuedPublicFunctionCalls[1].toCallRequest(),
    tx.enqueuedPublicFunctionCalls[0].toCallRequest(),
    CallRequest.empty(),
  ];

  tx.data.end.publicCallStack = makeTuple(
    MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    i => tx.enqueuedPublicFunctionCalls[i + 2]?.toCallRequest() ?? CallRequest.empty(),
  ).reverse() as Tuple<CallRequest, typeof MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>;

  return tx;
};

export const randomContractArtifact = (): ContractArtifact => ({
  name: randomBytes(4).toString('hex'),
  functions: [],
  events: [],
  fileMap: {},
});

export const randomContractInstanceWithAddress = (): ContractInstanceWithAddress =>
  SerializableContractInstance.random().withAddress(AztecAddress.random());

export const randomDeployedContract = (): DeployedContract => ({
  artifact: randomContractArtifact(),
  instance: randomContractInstanceWithAddress(),
});

export const randomExtendedNote = ({
  note = Note.random(),
  owner = AztecAddress.random(),
  contractAddress = AztecAddress.random(),
  txHash = randomTxHash(),
  storageSlot = Fr.random(),
  noteTypeId = Fr.random(),
}: Partial<ExtendedNote> = {}) => {
  return new ExtendedNote(note, owner, contractAddress, storageSlot, noteTypeId, txHash);
};
