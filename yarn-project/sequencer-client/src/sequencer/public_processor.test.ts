import { PublicExecution, PublicExecutionResult, PublicExecutor } from '@aztec/acir-simulator';
import {
  ARGS_LENGTH,
  CallContext,
  CircuitsWasm,
  EthAddress,
  Fr,
  FunctionData,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  Proof,
  PublicCallRequest,
  makeEmptyProof,
  makeTuple,
} from '@aztec/circuits.js';
import { computeCallStackItemHash } from '@aztec/circuits.js/abis';
import {
  makeAztecAddress,
  makeKernelPublicInputs,
  makePublicCallRequest,
  makeSelector,
} from '@aztec/circuits.js/factories';
import { padArrayEnd } from '@aztec/foundation/collection';
import { SiblingPath } from '@aztec/merkle-tree';
import {
  ContractDataSource,
  ContractPublicData,
  EncodedContractFunction,
  Tx,
  TxExecutionRequest,
  NoirLogs,
} from '@aztec/types';
import { MerkleTreeOperations, TreeInfo } from '@aztec/world-state';
import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';
import pick from 'lodash.pick';
import times from 'lodash.times';
import { makePrivateTx, makePublicTx } from '../index.js';
import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { WasmPublicKernelCircuitSimulator } from '../simulator/public_kernel.js';
import { PublicProcessor } from './public_processor.js';

describe('public_processor', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let publicExecutor: MockProxy<PublicExecutor>;
  let publicProver: MockProxy<PublicProver>;
  let contractDataSource: MockProxy<ContractDataSource>;

  let publicFunction: EncodedContractFunction;
  let contractData: ContractPublicData;
  let proof: Proof;
  let root: Buffer;

  let processor: PublicProcessor;

  beforeEach(() => {
    db = mock<MerkleTreeOperations>();
    publicExecutor = mock<PublicExecutor>();
    publicProver = mock<PublicProver>();
    contractDataSource = mock<ContractDataSource>();

    contractData = ContractPublicData.random();
    publicFunction = EncodedContractFunction.random();
    proof = makeEmptyProof();
    root = Buffer.alloc(32, 5);

    publicProver.getPublicCircuitProof.mockResolvedValue(proof);
    publicProver.getPublicKernelCircuitProof.mockResolvedValue(proof);
    db.getTreeInfo.mockResolvedValue({ root } as TreeInfo);
    contractDataSource.getL2ContractPublicData.mockResolvedValue(contractData);
    contractDataSource.getPublicFunction.mockResolvedValue(publicFunction);
  });

  describe('with mock circuits', () => {
    let publicKernel: MockProxy<PublicKernelCircuitSimulator>;

    beforeEach(() => {
      publicKernel = mock<PublicKernelCircuitSimulator>();
      processor = new PublicProcessor(db, publicExecutor, publicKernel, publicProver, contractDataSource);
    });

    it('skips non-public txs without public execution requests', async function () {
      const tx = makePrivateTx();
      tx.data.end.publicCallStack = makeTuple(KERNEL_PUBLIC_CALL_STACK_LENGTH, Fr.zero);
      const hash = await tx.getTxHash();
      const [processed, failed] = await processor.process([tx]);

      expect(processed).toEqual([{ isEmpty: false, hash, ...pick(tx, 'data', 'proof', 'encryptedLogs') }]);
      expect(failed).toEqual([]);
    });

    it('returns failed txs without aborting entire operation', async function () {
      publicExecutor.execute.mockRejectedValue(new Error(`Failed`));

      const tx = makePublicTx();
      const [processed, failed] = await processor.process([tx]);

      expect(processed).toEqual([]);
      expect(failed).toEqual([tx]);
    });

    it('runs a public tx', async function () {
      const tx = makePublicTx();
      const hash = await tx.getTxHash();

      const publicExecutionResult = makePublicExecutionResult(tx.txRequest.txRequest);
      publicExecutor.execute.mockResolvedValue(publicExecutionResult);

      const path = times(PUBLIC_DATA_TREE_HEIGHT, i => Buffer.alloc(32, i));
      db.getSiblingPath.mockResolvedValue(new SiblingPath<number>(PUBLIC_DATA_TREE_HEIGHT, path));

      const output = makeKernelPublicInputs();
      publicKernel.publicKernelCircuitNoInput.mockResolvedValue(output);

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([{ isEmpty: false, hash, data: output, proof, ...pick(tx, 'txRequest') }]);
      expect(failed).toEqual([]);

      expect(publicExecutor.execute).toHaveBeenCalled();
      expect(publicKernel.publicKernelCircuitNoInput).toHaveBeenCalled();
    });
  });

  describe('with actual circuits', () => {
    let publicKernel: PublicKernelCircuitSimulator;
    let wasm: CircuitsWasm;

    beforeAll(async () => {
      wasm = await CircuitsWasm.get();
    });

    beforeEach(() => {
      const path = times(PUBLIC_DATA_TREE_HEIGHT, i => Buffer.alloc(32, i));
      db.getSiblingPath.mockResolvedValue(new SiblingPath<number>(PUBLIC_DATA_TREE_HEIGHT, path));
      publicKernel = new WasmPublicKernelCircuitSimulator();
      processor = new PublicProcessor(db, publicExecutor, publicKernel, publicProver, contractDataSource);
    });

    const expectedProcessedTx = async (tx: Tx) =>
      expect.objectContaining({
        hash: await tx.getTxHash(),
        proof,
        txRequest: tx.txRequest,
        isEmpty: false,
        data: expect.objectContaining({ isPrivate: false }),
      });

    const expectedTxByHash = async (tx: Tx) =>
      expect.objectContaining({
        hash: await tx.getTxHash(),
        proof,
      });

    it('runs a public tx', async function () {
      const publicKernelSpy = jest.spyOn(publicKernel, 'publicKernelCircuitNoInput');
      const tx = makePublicTx(0x10);
      tx.txRequest.txRequest.functionData.isConstructor = false;
      tx.txRequest.txRequest.functionData.isPrivate = false;

      const publicExecutionResult = makePublicExecutionResult(tx.txRequest.txRequest);
      publicExecutor.execute.mockResolvedValue(publicExecutionResult);

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([await expectedProcessedTx(tx)]);
      expect(failed).toEqual([]);

      expect(publicExecutor.execute).toHaveBeenCalled();
      expect(publicKernelSpy).toHaveBeenCalled();
    });

    it('runs a public tx with nested execution', async function () {
      const tx = makePublicTx(0x10);
      const txRequest = tx.txRequest.txRequest;
      txRequest.functionData.isConstructor = false;
      txRequest.functionData.isPrivate = false;

      const publicExecutionResult = makePublicExecutionResult(txRequest);
      publicExecutionResult.nestedExecutions = [
        makePublicExecutionResult({
          from: txRequest.to,
          to: makeAztecAddress(30),
          functionData: new FunctionData(makeSelector(5), false, false),
          args: new Array(ARGS_LENGTH).fill(Fr.ZERO),
        }),
      ];
      publicExecutor.execute.mockResolvedValue(publicExecutionResult);

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([await expectedProcessedTx(tx)]);

      const publicCallStack = processed[0].data.end.publicCallStack;
      expect(publicCallStack).toEqual(times(KERNEL_PUBLIC_CALL_STACK_LENGTH, () => expect.any(Fr)));

      expect(failed).toEqual([]);
      expect(publicExecutor.execute).toHaveBeenCalled();
    });

    it('runs a public tx with nested execution two levels deep', async function () {
      const tx = makePublicTx(0x10);
      const txRequest = tx.txRequest.txRequest;
      txRequest.functionData.isConstructor = false;
      txRequest.functionData.isPrivate = false;

      const intermediateContractAddress = makeAztecAddress(30);
      const publicExecutionResult = makePublicExecutionResult(txRequest, [
        makePublicExecutionResult(
          {
            from: txRequest.to,
            to: intermediateContractAddress,
            functionData: new FunctionData(makeSelector(5), false, false),
            args: new Array(ARGS_LENGTH).fill(Fr.ZERO),
          },
          [
            makePublicExecutionResult({
              from: intermediateContractAddress,
              to: makeAztecAddress(40),
              functionData: new FunctionData(makeSelector(15), false, false),
              args: new Array(ARGS_LENGTH).fill(Fr.ZERO),
            }),
          ],
        ),
      ]);
      publicExecutor.execute.mockResolvedValue(publicExecutionResult);

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([await expectedProcessedTx(tx)]);
      expect(failed).toEqual([]);
      expect(publicExecutor.execute).toHaveBeenCalled();
    });

    it('runs a private tx with enqueued calls', async function () {
      const callRequests: PublicCallRequest[] = [makePublicCallRequest(0x100), makePublicCallRequest(0x100)];
      const callStackItems = await Promise.all(callRequests.map(call => call.toPublicCallStackItem()));
      const callStackHashes = callStackItems.map(call => computeCallStackItemHash(wasm, call));

      const kernelOutput = makeKernelPublicInputs(0x10);
      kernelOutput.end.publicCallStack = padArrayEnd(callStackHashes, Fr.ZERO, KERNEL_PUBLIC_CALL_STACK_LENGTH);
      kernelOutput.end.privateCallStack = padArrayEnd([], Fr.ZERO, KERNEL_PRIVATE_CALL_STACK_LENGTH);

      const tx = Tx.createPrivate(kernelOutput, proof, NoirLogs.random(2), [], callRequests);

      publicExecutor.execute.mockImplementation(execution => {
        for (const request of callRequests) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            return Promise.resolve(makePublicExecutionResultFromRequest(request));
          }
        }
        throw new Error(`Unexpected execution request: ${execution}`);
      });

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([await expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.execute).toHaveBeenCalledTimes(2);
    });

    it.todo('runs a private tx with enqueued calls that span nested calls');
  });
});

function makePublicExecutionResultFromRequest(item: PublicCallRequest): PublicExecutionResult {
  return {
    execution: item,
    nestedExecutions: [],
    returnValues: [new Fr(1n)],
    contractStorageReads: [],
    contractStorageUpdateRequests: [],
  };
}

function makePublicExecutionResult(
  tx: Pick<TxExecutionRequest, 'from' | 'to' | 'functionData' | 'args'>,
  nestedExecutions: PublicExecutionResult[] = [],
): PublicExecutionResult {
  const callContext = new CallContext(tx.from, tx.to, EthAddress.ZERO, false, false, false);
  const execution: PublicExecution = {
    callContext,
    contractAddress: tx.to,
    functionData: tx.functionData,
    args: tx.args,
  };
  return {
    execution,
    nestedExecutions,
    returnValues: [],
    contractStorageReads: [],
    contractStorageUpdateRequests: [],
  };
}
