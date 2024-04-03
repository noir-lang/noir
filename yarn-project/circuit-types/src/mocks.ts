import {
  AztecAddress,
  CallRequest,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  Proof,
  computeContractClassId,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import { type ContractArtifact, type DecodedReturn } from '@aztec/foundation/abi';
import { makeTuple } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { randomBytes } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';
import { type ContractInstanceWithAddress, SerializableContractInstance } from '@aztec/types/contracts';

import { EncryptedL2Log } from './logs/encrypted_l2_log.js';
import { EncryptedFunctionL2Logs, EncryptedTxL2Logs, Note, UnencryptedTxL2Logs } from './logs/index.js';
import { makePrivateKernelTailCircuitPublicInputs, makePublicCallRequest } from './mocks_to_purge.js';
import { ExtendedNote } from './notes/index.js';
import { SimulatedTx, Tx, TxHash } from './tx/index.js';

/**
 * Testing utility to create empty logs composed from a single empty log.
 */
export function makeEmptyLogs(): EncryptedTxL2Logs {
  const functionLogs = [new EncryptedFunctionL2Logs([EncryptedL2Log.empty()])];
  return new EncryptedTxL2Logs(functionLogs);
}

export const randomTxHash = (): TxHash => new TxHash(randomBytes(32));

export const mockTx = (seed = 1, logs = true) => {
  const tx = new Tx(
    makePrivateKernelTailCircuitPublicInputs(seed),
    new Proof(Buffer.alloc(0)),
    logs ? EncryptedTxL2Logs.random(8, 3) : EncryptedTxL2Logs.empty(), // 8 priv function invocations creating 3 encrypted logs each
    logs ? UnencryptedTxL2Logs.random(11, 2) : UnencryptedTxL2Logs.empty(), // 8 priv + 3 pub function invocations creating 2 unencrypted logs each
    times(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makePublicCallRequest),
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

export const mockSimulatedTx = (seed = 1, logs = true) => {
  const tx = mockTx(seed, logs);
  const dec: DecodedReturn = [1n, 2n, 3n, 4n];
  return new SimulatedTx(tx, dec, dec);
};

export const randomContractArtifact = (): ContractArtifact => ({
  name: randomBytes(4).toString('hex'),
  functions: [],
  events: [],
  fileMap: {},
});

export const randomContractInstanceWithAddress = (opts: { contractClassId?: Fr } = {}): ContractInstanceWithAddress =>
  SerializableContractInstance.random(opts).withAddress(AztecAddress.random());

export const randomDeployedContract = () => {
  const artifact = randomContractArtifact();
  const contractClassId = computeContractClassId(getContractClassFromArtifact(artifact));
  return { artifact, instance: randomContractInstanceWithAddress({ contractClassId }) };
};

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
