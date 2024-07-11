import { Note, type PrivateKernelProver } from '@aztec/circuit-types';
import {
  FunctionData,
  FunctionSelector,
  MAX_NOTE_HASHES_PER_CALL,
  MAX_NOTE_HASHES_PER_TX,
  MembershipWitness,
  NoteHash,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateKernelCircuitPublicInputs,
  PrivateKernelTailCircuitPublicInputs,
  PublicCallRequest,
  ScopedNoteHash,
  type TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  VerificationKeyAsFields,
} from '@aztec/circuits.js';
import { makeTxRequest } from '@aztec/circuits.js/testing';
import { NoteSelector } from '@aztec/foundation/abi';
import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type ExecutionResult, type NoteAndSlot } from '@aztec/simulator';

import { mock } from 'jest-mock-extended';

import { KernelProver } from './kernel_prover.js';
import { type ProvingDataOracle } from './proving_data_oracle.js';

describe('Kernel Prover', () => {
  let txRequest: TxRequest;
  let oracle: ReturnType<typeof mock<ProvingDataOracle>>;
  let proofCreator: ReturnType<typeof mock<PrivateKernelProver>>;
  let prover: KernelProver;
  let dependencies: { [name: string]: string[] } = {};

  const contractAddress = AztecAddress.fromBigInt(987654n);

  const notesAndSlots: NoteAndSlot[] = Array(10)
    .fill(null)
    .map(() => ({
      note: new Note([Fr.random(), Fr.random(), Fr.random()]),
      storageSlot: Fr.random(),
      noteTypeId: NoteSelector.random(),
      owner: { x: Fr.random(), y: Fr.random() },
    }));

  const createFakeSiloedCommitment = (commitment: Fr) => new Fr(commitment.value + 1n);
  const generateFakeCommitment = (noteAndSlot: NoteAndSlot) => noteAndSlot.note.items[0];
  const generateFakeSiloedCommitment = (note: NoteAndSlot) => createFakeSiloedCommitment(generateFakeCommitment(note));

  const createExecutionResult = (fnName: string, newNoteIndices: number[] = []): ExecutionResult => {
    const publicInputs = PrivateCircuitPublicInputs.empty();
    publicInputs.noteHashes = makeTuple(
      MAX_NOTE_HASHES_PER_CALL,
      i =>
        i < newNoteIndices.length
          ? new NoteHash(generateFakeCommitment(notesAndSlots[newNoteIndices[i]]), 0)
          : NoteHash.empty(),
      0,
    );
    const functionData = FunctionData.empty();
    functionData.selector = new FunctionSelector(fnName.charCodeAt(0));
    return {
      callStackItem: new PrivateCallStackItem(AztecAddress.ZERO, functionData, publicInputs),
      nestedExecutions: (dependencies[fnName] || []).map(name => createExecutionResult(name)),
      vk: VerificationKey.makeFake().toBuffer(),
      newNotes: newNoteIndices.map(idx => notesAndSlots[idx]),
      nullifiedNoteHashCounters: new Map(),
      noteHashLeafIndexMap: new Map(),
      returnValues: [],
      acir: Buffer.alloc(0),
      partialWitness: new Map(),
      enqueuedPublicFunctionCalls: [],
      publicTeardownFunctionCall: PublicCallRequest.empty(),
      noteEncryptedLogs: [],
      encryptedLogs: [],
      unencryptedLogs: [],
    };
  };

  const simulateProofOutput = (newNoteIndices: number[]) => {
    const publicInputs = PrivateKernelCircuitPublicInputs.empty();
    const noteHashes = makeTuple(MAX_NOTE_HASHES_PER_TX, ScopedNoteHash.empty);
    for (let i = 0; i < newNoteIndices.length; i++) {
      noteHashes[i] = new NoteHash(generateFakeSiloedCommitment(notesAndSlots[newNoteIndices[i]]), 0).scope(
        contractAddress,
      );
    }

    publicInputs.end.noteHashes = noteHashes;
    return {
      publicInputs,
      verificationKey: VerificationKeyAsFields.makeEmpty(),
      outputWitness: new Map(),
    };
  };

  const simulateProofOutputFinal = (newNoteIndices: number[]) => {
    const publicInputs = PrivateKernelTailCircuitPublicInputs.empty();
    const noteHashes = makeTuple(MAX_NOTE_HASHES_PER_TX, () => Fr.ZERO);
    for (let i = 0; i < newNoteIndices.length; i++) {
      noteHashes[i] = generateFakeSiloedCommitment(notesAndSlots[newNoteIndices[i]]);
    }
    publicInputs.forRollup!.end.noteHashes = noteHashes;

    return {
      publicInputs,
      outputWitness: new Map(),
      verificationKey: VerificationKeyAsFields.makeEmpty(),
    };
  };

  const computeAppCircuitVerificationKeyOutput = () => {
    return {
      verificationKey: VerificationKeyAsFields.makeEmpty(),
    };
  };

  const expectExecution = (fns: string[]) => {
    const callStackItemsInit = proofCreator.simulateProofInit.mock.calls.map(args =>
      String.fromCharCode(args[0].privateCall.callStackItem.functionData.selector.value),
    );
    const callStackItemsInner = proofCreator.simulateProofInner.mock.calls.map(args =>
      String.fromCharCode(args[0].privateCall.callStackItem.functionData.selector.value),
    );

    expect(proofCreator.simulateProofInit).toHaveBeenCalledTimes(Math.min(1, fns.length));
    expect(proofCreator.simulateProofInner).toHaveBeenCalledTimes(Math.max(0, fns.length - 1));
    expect(callStackItemsInit.concat(callStackItemsInner)).toEqual(fns);
    proofCreator.simulateProofInner.mockClear();
    proofCreator.simulateProofInit.mockClear();
  };

  const prove = (executionResult: ExecutionResult) => prover.prove(txRequest, executionResult);

  beforeEach(() => {
    txRequest = makeTxRequest();

    oracle = mock<ProvingDataOracle>();
    // TODO(dbanks12): will need to mock oracle.getNoteMembershipWitness() to test non-transient reads
    oracle.getVkMembershipWitness.mockResolvedValue(MembershipWitness.random(VK_TREE_HEIGHT));

    oracle.getContractAddressPreimage.mockResolvedValue({
      contractClassId: Fr.random(),
      publicKeysHash: Fr.random(),
      saltedInitializationHash: Fr.random(),
    });
    oracle.getContractClassIdPreimage.mockResolvedValue({
      artifactHash: Fr.random(),
      publicBytecodeCommitment: Fr.random(),
      privateFunctionsRoot: Fr.random(),
    });

    proofCreator = mock<PrivateKernelProver>();
    proofCreator.getSiloedCommitments.mockImplementation(publicInputs =>
      Promise.resolve(publicInputs.noteHashes.map(com => createFakeSiloedCommitment(com.value))),
    );
    proofCreator.simulateProofInit.mockResolvedValue(simulateProofOutput([]));
    proofCreator.simulateProofInner.mockResolvedValue(simulateProofOutput([]));
    proofCreator.simulateProofReset.mockResolvedValue(simulateProofOutput([]));
    proofCreator.simulateProofTail.mockResolvedValue(simulateProofOutputFinal([]));
    proofCreator.computeAppCircuitVerificationKey.mockResolvedValue(computeAppCircuitVerificationKeyOutput());

    prover = new KernelProver(oracle, proofCreator);
  });

  it('should create proofs in correct order', async () => {
    {
      dependencies = { a: [] };
      const executionResult = createExecutionResult('a');
      await prove(executionResult);
      expectExecution(['a']);
    }

    {
      dependencies = {
        a: ['b', 'd'],
        b: ['c'],
      };
      const executionResult = createExecutionResult('a');
      await prove(executionResult);
      expectExecution(['a', 'b', 'c', 'd']);
    }

    {
      dependencies = {
        k: ['m', 'o'],
        m: ['q'],
        o: ['n', 'p', 'r'],
      };
      const executionResult = createExecutionResult('k');
      await prove(executionResult);
      expectExecution(['k', 'm', 'q', 'o', 'n', 'p', 'r']);
    }
  });
});
