import { ExecutionResult, NoteAndSlot } from '@aztec/acir-simulator';
import { FunctionL2Logs, Note } from '@aztec/circuit-types';
import {
  FunctionData,
  FunctionSelector,
  KernelCircuitPublicInputs,
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_READ_REQUESTS_PER_CALL,
  MembershipWitness,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  ReadRequestMembershipWitness,
  SideEffect,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTxRequest } from '@aztec/circuits.js/factories';
import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { Tuple } from '@aztec/foundation/serialize';

import { mock } from 'jest-mock-extended';

import { KernelProver, OutputNoteData } from './kernel_prover.js';
import { ProofCreator } from './proof_creator.js';
import { ProvingDataOracle } from './proving_data_oracle.js';

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
      owner: { x: Fr.random(), y: Fr.random() },
    }));

  const createFakeSiloedCommitment = (commitment: Fr) => new Fr(commitment.value + 1n);
  const generateFakeCommitment = (noteAndSlot: NoteAndSlot) => noteAndSlot.note.items[0];
  const generateFakeSiloedCommitment = (note: NoteAndSlot) => createFakeSiloedCommitment(generateFakeCommitment(note));

  const createExecutionResult = (fnName: string, newNoteIndices: number[] = []): ExecutionResult => {
    const publicInputs = PrivateCircuitPublicInputs.empty();
    publicInputs.newCommitments = makeTuple(
      MAX_NEW_COMMITMENTS_PER_CALL,
      i =>
        i < newNoteIndices.length
          ? new SideEffect(generateFakeCommitment(notesAndSlots[newNoteIndices[i]]), Fr.ZERO)
          : SideEffect.empty(),
      0,
    );
    const functionData = FunctionData.empty();
    functionData.selector = new FunctionSelector(fnName.charCodeAt(0));
    return {
      callStackItem: new PrivateCallStackItem(AztecAddress.ZERO, functionData, publicInputs, false),
      nestedExecutions: (dependencies[fnName] || []).map(name => createExecutionResult(name)),
      vk: VerificationKey.makeFake().toBuffer(),
      newNotes: newNoteIndices.map(idx => notesAndSlots[idx]),
      // TODO(dbanks12): should test kernel prover with non-transient reads.
      // This will be necessary once kernel actually checks (attempts to match) transient reads.
      readRequestPartialWitnesses: Array.from({ length: MAX_READ_REQUESTS_PER_CALL }, () =>
        ReadRequestMembershipWitness.emptyTransient(),
      ),
      returnValues: [],
      acir: Buffer.alloc(0),
      partialWitness: new Map(),
      enqueuedPublicFunctionCalls: [],
      encryptedLogs: new FunctionL2Logs([]),
      unencryptedLogs: new FunctionL2Logs([]),
    };
  };

  const createProofOutput = (newNoteIndices: number[]) => {
    const publicInputs = KernelCircuitPublicInputs.empty();
    const commitments = newNoteIndices.map(
      idx => new SideEffect(generateFakeSiloedCommitment(notesAndSlots[idx]), Fr.ZERO),
    );
    // TODO(AD) FIXME(AD) This cast is bad. Why is this not the correct length when this is called?
    publicInputs.end.newCommitments = commitments as Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_TX>;
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

  const expectOutputNotes = (outputNotes: OutputNoteData[], expectedNoteIndices: number[]) => {
    expect(outputNotes.length).toBe(expectedNoteIndices.length);
    outputNotes.forEach((n, i) => {
      expect(n.data).toEqual(notesAndSlots[expectedNoteIndices[i]]);
    });
  };

  const prove = (executionResult: ExecutionResult) => prover.prove(txRequest, executionResult);

  beforeEach(() => {
    txRequest = makeTxRequest();

    oracle = mock<ProvingDataOracle>();
    // TODO(dbanks12): will need to mock oracle.getNoteMembershipWitness() to test non-transient reads
    oracle.getVkMembershipWitness.mockResolvedValue(MembershipWitness.random(VK_TREE_HEIGHT));

    proofCreator = mock<ProofCreator>();
    proofCreator.getSiloedCommitments.mockImplementation(publicInputs =>
      Promise.resolve(publicInputs.newCommitments.map(com => createFakeSiloedCommitment(com.value))),
    );
    proofCreator.createProofInit.mockResolvedValue(createProofOutput([]));
    proofCreator.createProofInner.mockResolvedValue(createProofOutput([]));
    proofCreator.createProofOrdering.mockResolvedValue(createProofOutput([]));

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

  it('should only return notes that are outputted from the final proof', async () => {
    const resultA = createExecutionResult('a', [1, 2, 3]);
    const resultB = createExecutionResult('b', [4]);
    const resultC = createExecutionResult('c', [5, 6]);
    proofCreator.createProofInit.mockResolvedValueOnce(createProofOutput([1, 2, 3]));
    proofCreator.createProofInner.mockResolvedValueOnce(createProofOutput([1, 3, 4]));
    proofCreator.createProofInner.mockResolvedValueOnce(createProofOutput([1, 3, 5, 6]));
    proofCreator.createProofOrdering.mockResolvedValueOnce(createProofOutput([1, 3, 5, 6]));

    const executionResult = {
      ...resultA,
      nestedExecutions: [resultB, resultC],
    };
    const { outputNotes } = await prove(executionResult);
    expectExecution(['a', 'c', 'b']);
    expectOutputNotes(outputNotes, [1, 3, 5, 6]);
  });
});
