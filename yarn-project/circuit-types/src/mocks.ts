import {
  AztecAddress,
  CallRequest,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  PartialPrivateTailPublicInputsForPublic,
  PrivateKernelTailCircuitPublicInputs,
  Proof,
  SideEffectLinkedToNoteHash,
  computeContractClassId,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import { makePublicCallRequest } from '@aztec/circuits.js/testing';
import { type ContractArtifact, type DecodedReturn } from '@aztec/foundation/abi';
import { makeTuple } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { randomBytes } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type ContractInstanceWithAddress, SerializableContractInstance } from '@aztec/types/contracts';

import { EncryptedL2Log } from './logs/encrypted_l2_log.js';
import { EncryptedFunctionL2Logs, EncryptedTxL2Logs, Note, UnencryptedTxL2Logs } from './logs/index.js';
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

export const mockTx = (
  seed = 1,
  {
    hasLogs = false,
    numberOfNonRevertiblePublicCallRequests = MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX / 2,
    numberOfRevertiblePublicCallRequests = MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX / 2,
  }: {
    hasLogs?: boolean;
    numberOfNonRevertiblePublicCallRequests?: number;
    numberOfRevertiblePublicCallRequests?: number;
  } = {},
) => {
  const totalPublicCallRequests = numberOfNonRevertiblePublicCallRequests + numberOfRevertiblePublicCallRequests;
  const publicCallRequests = times(totalPublicCallRequests, i => makePublicCallRequest(seed + 0x100 + i));

  const isForPublic = totalPublicCallRequests > 0;
  const data = PrivateKernelTailCircuitPublicInputs.empty();
  const firstNullifier = new SideEffectLinkedToNoteHash(new Fr(seed), new Fr(seed + 1), Fr.ZERO);

  if (isForPublic) {
    data.forRollup = undefined;
    data.forPublic = PartialPrivateTailPublicInputsForPublic.empty();

    data.forPublic.endNonRevertibleData.newNullifiers[0] = firstNullifier;

    data.forPublic.endNonRevertibleData.publicCallStack = makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, i =>
      i < numberOfNonRevertiblePublicCallRequests ? publicCallRequests[i].toCallRequest() : CallRequest.empty(),
    );

    data.forPublic.end.publicCallStack = makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, i =>
      i < numberOfRevertiblePublicCallRequests
        ? publicCallRequests[i + numberOfNonRevertiblePublicCallRequests].toCallRequest()
        : CallRequest.empty(),
    );
  } else {
    data.forRollup!.end.newNullifiers[0] = firstNullifier;
  }

  const target = isForPublic ? data.forPublic! : data.forRollup!;

  const encryptedLogs = hasLogs ? EncryptedTxL2Logs.random(8, 3) : EncryptedTxL2Logs.empty(); // 8 priv function invocations creating 3 encrypted logs each
  const unencryptedLogs = hasLogs ? UnencryptedTxL2Logs.random(11, 2) : UnencryptedTxL2Logs.empty(); // 8 priv function invocations creating 3 encrypted logs each
  if (!hasLogs) {
    target.end.encryptedLogsHash = Fr.ZERO;
    target.end.unencryptedLogsHash = Fr.ZERO;
  } else {
    target.end.encryptedLogsHash = Fr.fromBuffer(encryptedLogs.hash());
    target.end.unencryptedLogsHash = Fr.fromBuffer(unencryptedLogs.hash());
  }

  const tx = new Tx(data, new Proof(Buffer.alloc(0)), encryptedLogs, unencryptedLogs, publicCallRequests);

  return tx;
};

export const mockTxForRollup = (seed = 1, { hasLogs = false }: { hasLogs?: boolean } = {}) =>
  mockTx(seed, { hasLogs, numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 0 });

export const mockSimulatedTx = (seed = 1, hasLogs = true) => {
  const tx = mockTx(seed, { hasLogs });
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
