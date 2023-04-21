import { ExecutionResult, NewNoteData } from '@aztec/acir-simulator';
import {
  EcdsaSignature,
  MembershipWitness,
  PRIVATE_CALL_STACK_LENGTH,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  KernelCircuitPublicInputs,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTxRequest } from '@aztec/circuits.js/factories';
import { AztecAddress, Fr } from '@aztec/foundation';
import { mock } from 'jest-mock-extended';
import { KernelProver, OutputNoteData } from './kernel_prover.js';
import { ProofCreator } from './proof_creator.js';
import { ProvingDataOracle } from './proving_data_oracle.js';

describe('Kernel Prover', () => {
  let txRequest: TxRequest;
  let txSignature: EcdsaSignature;
  let oracle: ReturnType<typeof mock<ProvingDataOracle>>;
  let proofCreator: ReturnType<typeof mock<ProofCreator>>;
  let prover: KernelProver;
  let dependencies: { [name: string]: string[] } = {};

  const notes: NewNoteData[] = Array(10)
    .fill(null)
    .map(() => ({
      preimage: [Fr.random(), Fr.random(), Fr.random()],
      storageSlot: Fr.random(),
      owner: { x: Fr.random(), y: Fr.random() },
    }));

  const createFakeSiloedCommitment = (commitment: Fr) => new Fr(commitment.value + 1n);
  const generateFakeCommitment = (note: NewNoteData) => note.preimage[0];
  const generateFakeSiloedCommitment = (note: NewNoteData) => createFakeSiloedCommitment(generateFakeCommitment(note));

  const createExecutionResult = (fnName: string, newNoteIndices: number[] = []): ExecutionResult => {
    const publicInputs = PrivateCircuitPublicInputs.empty();
    publicInputs.newCommitments = newNoteIndices.map(idx => generateFakeCommitment(notes[idx]));
    return {
      // Replace `FunctionData` with `string` for easier testing.
      callStackItem: new PrivateCallStackItem(AztecAddress.ZERO, fnName as any, publicInputs),
      nestedExecutions: (dependencies[fnName] || []).map(name => createExecutionResult(name)),
      vk: VerificationKey.makeFake().toBuffer(),
      preimages: { newNotes: newNoteIndices.map(idx => notes[idx]), nullifiedNotes: [] },
      returnValues: [],
      acir: Buffer.alloc(0),
      partialWitness: new Map(),
    } as ExecutionResult;
  };

  const createProofOutput = (newNoteIndices: number[]) => {
    const publicInputs = KernelCircuitPublicInputs.empty();
    publicInputs.end.newCommitments = newNoteIndices.map(idx => generateFakeSiloedCommitment(notes[idx]));
    return {
      publicInputs,
      proof: makeEmptyProof(),
    };
  };

  const expectExecution = (fns: string[]) => {
    const callStackItems = proofCreator.createProof.mock.calls.map(args => args[2].callStackItem.functionData);
    expect(callStackItems).toEqual(fns);
    proofCreator.createProof.mockClear();
  };

  const expectOutputNotes = (outputNotes: OutputNoteData[], expectedNoteIndices: number[]) => {
    expect(outputNotes.length).toBe(expectedNoteIndices.length);
    outputNotes.forEach((n, i) => {
      expect(n.data).toEqual(notes[expectedNoteIndices[i]]);
    });
  };

  const prove = (executionResult: ExecutionResult) => prover.prove(txRequest, txSignature, executionResult);

  beforeEach(() => {
    txRequest = makeTxRequest();
    txSignature = EcdsaSignature.random();

    oracle = mock<ProvingDataOracle>();
    oracle.getVkMembershipWitness.mockResolvedValue(MembershipWitness.random(VK_TREE_HEIGHT));

    proofCreator = mock<ProofCreator>();
    proofCreator.getSiloedCommitments.mockImplementation(publicInputs =>
      Promise.resolve(publicInputs.newCommitments.map(createFakeSiloedCommitment)),
    );
    proofCreator.createProof.mockResolvedValue(createProofOutput([]));

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

  it('should throw if call stack is too deep', async () => {
    dependencies.a = Array(PRIVATE_CALL_STACK_LENGTH + 1)
      .fill(0)
      .map((_, i) => `${i}`);
    const executionResult = createExecutionResult('a');
    await expect(prove(executionResult)).rejects.toThrow();
  });

  it('should only return notes that are outputted from the final proof', async () => {
    const resultA = createExecutionResult('a', [1, 2, 3]);
    const resultB = createExecutionResult('b', [4]);
    const resultC = createExecutionResult('c', [5, 6]);
    proofCreator.createProof.mockResolvedValueOnce(createProofOutput([1, 2, 3]));
    proofCreator.createProof.mockResolvedValueOnce(createProofOutput([1, 3, 4]));
    proofCreator.createProof.mockResolvedValueOnce(createProofOutput([1, 3, 5, 6]));

    const executionResult = {
      ...resultA,
      nestedExecutions: [resultB, resultC],
    };
    const { outputNotes } = await prove(executionResult);
    expectExecution(['a', 'c', 'b']);
    expectOutputNotes(outputNotes, [1, 3, 5, 6]);
  });
});
