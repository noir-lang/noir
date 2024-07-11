import {
  AztecAddress,
  CallRequest,
  ClientIvcProof,
  GasSettings,
  LogHash,
  MAX_NULLIFIERS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  Nullifier,
  PartialPrivateTailPublicInputsForPublic,
  PrivateKernelTailCircuitPublicInputs,
  PublicAccumulatedDataBuilder,
  PublicCallRequest,
  computeContractClassId,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import {
  makeCombinedAccumulatedData,
  makeCombinedConstantData,
  makePublicCallRequest,
} from '@aztec/circuits.js/testing';
import { type ContractArtifact, NoteSelector } from '@aztec/foundation/abi';
import { makeTuple } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { randomBytes } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type ContractInstanceWithAddress, SerializableContractInstance } from '@aztec/types/contracts';

import { EncryptedNoteTxL2Logs, EncryptedTxL2Logs, Note, UnencryptedTxL2Logs } from './logs/index.js';
import { ExtendedNote } from './notes/index.js';
import { NestedProcessReturnValues, PublicSimulationOutput, SimulatedTx, Tx, TxHash } from './tx/index.js';

export const randomTxHash = (): TxHash => new TxHash(randomBytes(32));

export const mockTx = (
  seed = 1,
  {
    hasLogs = false,
    numberOfNonRevertiblePublicCallRequests = MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX / 2,
    numberOfRevertiblePublicCallRequests = MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX / 2,
    publicCallRequests = [],
    publicTeardownCallRequest = PublicCallRequest.empty(),
    feePayer = AztecAddress.ZERO,
  }: {
    hasLogs?: boolean;
    numberOfNonRevertiblePublicCallRequests?: number;
    numberOfRevertiblePublicCallRequests?: number;
    publicCallRequests?: PublicCallRequest[];
    publicTeardownCallRequest?: PublicCallRequest;
    feePayer?: AztecAddress;
  } = {},
) => {
  const totalPublicCallRequests =
    numberOfNonRevertiblePublicCallRequests + numberOfRevertiblePublicCallRequests || publicCallRequests.length;
  if (publicCallRequests.length && publicCallRequests.length !== totalPublicCallRequests) {
    throw new Error(
      `Provided publicCallRequests does not match the required number of call requests. Expected ${totalPublicCallRequests}. Got ${publicCallRequests.length}`,
    );
  }

  const isForPublic = totalPublicCallRequests > 0 || publicTeardownCallRequest.isEmpty() === false;
  const data = PrivateKernelTailCircuitPublicInputs.empty();
  const firstNullifier = new Nullifier(new Fr(seed + 1), 0, Fr.ZERO);
  const noteEncryptedLogs = EncryptedNoteTxL2Logs.empty(); // Mock seems to have no new notes => no note logs
  const encryptedLogs = hasLogs ? EncryptedTxL2Logs.random(2, 3) : EncryptedTxL2Logs.empty(); // 2 priv function invocations creating 3 encrypted logs each
  const unencryptedLogs = hasLogs ? UnencryptedTxL2Logs.random(2, 1) : UnencryptedTxL2Logs.empty(); // 2 priv function invocations creating 1 unencrypted log each
  data.constants.txContext.gasSettings = GasSettings.default();
  data.feePayer = feePayer;

  if (isForPublic) {
    data.forRollup = undefined;
    data.forPublic = PartialPrivateTailPublicInputsForPublic.empty();

    publicCallRequests = publicCallRequests.length
      ? publicCallRequests.slice().sort((a, b) => b.sideEffectCounter - a.sideEffectCounter)
      : times(totalPublicCallRequests, i => makePublicCallRequest(seed + 0x100 + i));

    const revertibleBuilder = new PublicAccumulatedDataBuilder();
    const nonRevertibleBuilder = new PublicAccumulatedDataBuilder();

    const nonRevertibleNullifiers = makeTuple(MAX_NULLIFIERS_PER_TX, Nullifier.empty);
    nonRevertibleNullifiers[0] = firstNullifier;

    data.forPublic.endNonRevertibleData = nonRevertibleBuilder
      .withNullifiers(nonRevertibleNullifiers)
      .withPublicCallStack(
        makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, i =>
          i < numberOfNonRevertiblePublicCallRequests
            ? publicCallRequests[numberOfRevertiblePublicCallRequests + i].toCallRequest()
            : CallRequest.empty(),
        ),
      )
      .build();

    data.forPublic.end = revertibleBuilder
      .withPublicCallStack(
        makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, i =>
          i < numberOfRevertiblePublicCallRequests ? publicCallRequests[i].toCallRequest() : CallRequest.empty(),
        ),
      )
      .build();

    data.forPublic.publicTeardownCallStack = makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, () => CallRequest.empty());
    data.forPublic.publicTeardownCallStack[0] = publicTeardownCallRequest.isEmpty()
      ? CallRequest.empty()
      : publicTeardownCallRequest.toCallRequest();

    if (hasLogs) {
      let i = 1; // 0 used in first nullifier
      let nonRevertibleIndex = 0;
      let revertibleIndex = 0;
      let functionCount = 0;
      encryptedLogs.functionLogs.forEach(functionLog => {
        functionLog.logs.forEach(log => {
          // ts complains if we dont check .forPublic here, even though it is defined ^
          if (data.forPublic) {
            const hash = new LogHash(
              Fr.fromBuffer(log.getSiloedHash()),
              i++,
              // +4 for encoding the length of the buffer
              new Fr(log.length + 4),
            );
            // make the first log non-revertible
            if (functionCount === 0) {
              data.forPublic.endNonRevertibleData.encryptedLogsHashes[nonRevertibleIndex++] = hash;
            } else {
              data.forPublic.end.encryptedLogsHashes[revertibleIndex++] = hash;
            }
          }
        });
        functionCount++;
      });
      nonRevertibleIndex = 0;
      revertibleIndex = 0;
      functionCount = 0;
      unencryptedLogs.functionLogs.forEach(functionLog => {
        functionLog.logs.forEach(log => {
          if (data.forPublic) {
            const hash = new LogHash(
              Fr.fromBuffer(log.getSiloedHash()),
              i++,
              // +4 for encoding the length of the buffer
              new Fr(log.length + 4),
            );
            // make the first log non-revertible
            if (functionCount === 0) {
              data.forPublic.endNonRevertibleData.unencryptedLogsHashes[nonRevertibleIndex++] = hash;
            } else {
              data.forPublic.end.unencryptedLogsHashes[revertibleIndex++] = hash;
            }
          }
        });
        functionCount++;
      });
    }
  } else {
    data.forRollup!.end.nullifiers[0] = firstNullifier.value;
    data.forRollup!.end.noteEncryptedLogsHash = Fr.fromBuffer(noteEncryptedLogs.hash());
    data.forRollup!.end.encryptedLogsHash = Fr.fromBuffer(encryptedLogs.hash());
    data.forRollup!.end.unencryptedLogsHash = Fr.fromBuffer(unencryptedLogs.hash());
  }

  const tx = new Tx(
    data,
    ClientIvcProof.empty(),
    noteEncryptedLogs,
    encryptedLogs,
    unencryptedLogs,
    publicCallRequests,
    publicTeardownCallRequest,
  );

  return tx;
};

export const mockTxForRollup = (seed = 1, { hasLogs = false }: { hasLogs?: boolean } = {}) =>
  mockTx(seed, { hasLogs, numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 0 });

export const mockSimulatedTx = (seed = 1, hasLogs = true) => {
  const tx = mockTx(seed, { hasLogs });
  const dec = new NestedProcessReturnValues([new Fr(1n), new Fr(2n), new Fr(3n), new Fr(4n)]);
  const output = new PublicSimulationOutput(
    tx.encryptedLogs,
    tx.unencryptedLogs,
    undefined,
    makeCombinedConstantData(),
    makeCombinedAccumulatedData(),
    [dec],
    {},
  );
  return new SimulatedTx(tx, dec, output);
};

export const randomContractArtifact = (): ContractArtifact => ({
  name: randomBytes(4).toString('hex'),
  functions: [],
  outputs: {
    structs: {},
    globals: {},
  },
  fileMap: {},
  storageLayout: {},
  notes: {},
});

export const randomContractInstanceWithAddress = (
  opts: { contractClassId?: Fr } = {},
  address: AztecAddress = AztecAddress.random(),
): ContractInstanceWithAddress => SerializableContractInstance.random(opts).withAddress(address);

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
  noteTypeId = NoteSelector.random(),
}: Partial<ExtendedNote> = {}) => {
  return new ExtendedNote(note, owner, contractAddress, storageSlot, noteTypeId, txHash);
};
