import { EncryptedFunctionL2Logs, Note, UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  FunctionData,
  FunctionSelector,
  MAX_NEW_NOTE_HASHES_PER_CALL,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
  MembershipWitness,
  NoteHashReadRequestMembershipWitness,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateKernelCircuitPublicInputs,
  PrivateKernelTailCircuitPublicInputs,
  SideEffect,
  type TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTxRequest } from '@aztec/circuits.js/testing';
import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type ExecutionResult, type NoteAndSlot } from '@aztec/simulator';

import { mock } from 'jest-mock-extended';

import { KernelProver } from './kernel_prover.js';
import { type ProofCreator } from './proof_creator.js';
import { type ProvingDataOracle } from './proving_data_oracle.js';

describe('Kernel Prover', () => {
  let txRequest: TxRequest;
  let oracle: ReturnType<typeof mock<ProvingDataOracle>>;
  let proofCreator: ReturnType<typeof mock<ProofCreator>>;
  let prover: KernelProver;
  let dependencies: { [name: string]: string[] } = {};

  const notesAndSlots: NoteAndSlot[] = Array(10)
    .fill(null)
    .map(() => ({
      note: new Note([Fr.random(), Fr.random(), Fr.random()]),
      storageSlot: Fr.random(),
      noteTypeId: Fr.random(),
      owner: { x: Fr.random(), y: Fr.random() },
    }));

  const createFakeSiloedCommitment = (commitment: Fr) => new Fr(commitment.value + 1n);
  const generateFakeCommitment = (noteAndSlot: NoteAndSlot) => noteAndSlot.note.items[0];
  const generateFakeSiloedCommitment = (note: NoteAndSlot) => createFakeSiloedCommitment(generateFakeCommitment(note));

  const createExecutionResult = (fnName: string, newNoteIndices: number[] = []): ExecutionResult => {
    const publicInputs = PrivateCircuitPublicInputs.empty();
    publicInputs.newNoteHashes = makeTuple(
      MAX_NEW_NOTE_HASHES_PER_CALL,
      i =>
        i < newNoteIndices.length
          ? new SideEffect(generateFakeCommitment(notesAndSlots[newNoteIndices[i]]), Fr.ZERO)
          : SideEffect.empty(),
      0,
    );
    const functionData = FunctionData.empty();
    functionData.selector = new FunctionSelector(fnName.charCodeAt(0));
    return {
      callStackItem: new PrivateCallStackItem(AztecAddress.ZERO, functionData, publicInputs),
      nestedExecutions: (dependencies[fnName] || []).map(name => createExecutionResult(name)),
      vk: VerificationKey.makeFake().toBuffer(),
      newNotes: newNoteIndices.map(idx => notesAndSlots[idx]),
      // TODO(dbanks12): should test kernel prover with non-transient reads.
      // This will be necessary once kernel actually checks (attempts to match) transient reads.
      noteHashReadRequestPartialWitnesses: Array.from({ length: MAX_NOTE_HASH_READ_REQUESTS_PER_CALL }, () =>
        NoteHashReadRequestMembershipWitness.emptyTransient(),
      ),
      returnValues: [],
      acir: Buffer.alloc(0),
      partialWitness: new Map(),
      enqueuedPublicFunctionCalls: [],
      encryptedLogs: EncryptedFunctionL2Logs.empty(),
      unencryptedLogs: UnencryptedFunctionL2Logs.empty(),
    };
  };

  const createProofOutput = (newNoteIndices: number[]) => {
    const publicInputs = PrivateKernelCircuitPublicInputs.empty();
    const commitments = makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, () => SideEffect.empty());
    for (let i = 0; i < newNoteIndices.length; i++) {
      commitments[i] = new SideEffect(generateFakeSiloedCommitment(notesAndSlots[newNoteIndices[i]]), Fr.ZERO);
    }

    publicInputs.end.newNoteHashes = commitments;
    return {
      publicInputs,
      proof: makeEmptyProof(),
    };
  };

  const createProofOutputFinal = (newNoteIndices: number[]) => {
    const publicInputs = PrivateKernelTailCircuitPublicInputs.empty();
    const commitments = makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, () => SideEffect.empty());
    for (let i = 0; i < newNoteIndices.length; i++) {
      commitments[i] = new SideEffect(generateFakeSiloedCommitment(notesAndSlots[newNoteIndices[i]]), Fr.ZERO);
    }

    publicInputs.forRollup!.end.newNoteHashes = commitments;

    return {
      publicInputs,
      proof: makeEmptyProof(),
    };
  };

  const expectExecution = (fns: string[]) => {
    const callStackItemsInit = proofCreator.createProofInit.mock.calls.map(args =>
      String.fromCharCode(args[0].privateCall.callStackItem.functionData.selector.value),
    );
    const callStackItemsInner = proofCreator.createProofInner.mock.calls.map(args =>
      String.fromCharCode(args[0].privateCall.callStackItem.functionData.selector.value),
    );

    expect(proofCreator.createProofInit).toHaveBeenCalledTimes(Math.min(1, fns.length));
    expect(proofCreator.createProofInner).toHaveBeenCalledTimes(Math.max(0, fns.length - 1));
    expect(callStackItemsInit.concat(callStackItemsInner)).toEqual(fns);
    proofCreator.createProofInner.mockClear();
    proofCreator.createProofInit.mockClear();
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

    proofCreator = mock<ProofCreator>();
    proofCreator.getSiloedCommitments.mockImplementation(publicInputs =>
      Promise.resolve(publicInputs.newNoteHashes.map(com => createFakeSiloedCommitment(com.value))),
    );
    proofCreator.createProofInit.mockResolvedValue(createProofOutput([]));
    proofCreator.createProofInner.mockResolvedValue(createProofOutput([]));
    proofCreator.createProofTail.mockResolvedValue(createProofOutputFinal([]));

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
      expectExecution(['a', 'd', 'b', 'c']);
    }

    {
      dependencies = {
        k: ['m', 'o'],
        m: ['q'],
        o: ['n', 'p', 'r'],
      };
      const executionResult = createExecutionResult('k');
      await prove(executionResult);
      expectExecution(['k', 'o', 'r', 'p', 'n', 'm', 'q']);
    }
  });
});
