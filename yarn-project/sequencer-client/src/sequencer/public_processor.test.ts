import { PublicExecution, PublicExecutionResult, PublicExecutor } from '@aztec/acir-simulator';
import {
  ARGS_LENGTH,
  CallContext,
  EthAddress,
  Fr,
  FunctionData,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  TxRequest,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeAztecAddress, makeKernelPublicInputs, makeSelector } from '@aztec/circuits.js/factories';
import { SiblingPath } from '@aztec/merkle-tree';
import { ContractDataSource, ContractPublicData, EncodedContractFunction, Tx } from '@aztec/types';
import { MerkleTreeOperations, TreeInfo } from '@aztec/world-state';
import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';
import pick from 'lodash.pick';
import times from 'lodash.times';
import { makePrivateTx, makePublicTx } from '../index.js';
import { Proof, PublicProver } from '../prover/index.js';
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

    it('skips non-public txs', async function () {
      const tx = makePrivateTx();
      const hash = await tx.getTxHash();
      const [processed, failed] = await processor.process([tx]);

      expect(processed).toEqual([{ isEmpty: false, hash, ...pick(tx, 'data', 'proof', 'unverifiedData') }]);
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
      db.getSiblingPath.mockResolvedValue(new SiblingPath(path));

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

    beforeEach(() => {
      const path = times(PUBLIC_DATA_TREE_HEIGHT, i => Buffer.alloc(32, i));
      db.getSiblingPath.mockResolvedValue(new SiblingPath(path));

      publicKernel = new WasmPublicKernelCircuitSimulator();
      processor = new PublicProcessor(db, publicExecutor, publicKernel, publicProver, contractDataSource);
    });

    const expectedProcessedTx = async (tx: Tx) =>
      expect.objectContaining({
        hash: await tx.getTxHash(),
        proof,
        txRequest: tx.txRequest,
        isEmpty: false,
        data: expect.objectContaining({ isPrivateKernel: false }),
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

      const callStackTop = publicCallStack[publicCallStack.length - 1];
      const callStackNext = publicCallStack[publicCallStack.length - 2];
      expect(callStackTop).not.toEqual(callStackNext);
      expect(publicCallStack.slice(0, KERNEL_PUBLIC_CALL_STACK_LENGTH - 1)).toEqual(
        times(KERNEL_PUBLIC_CALL_STACK_LENGTH - 1, () => callStackNext),
      );

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
  });
});

function makePublicExecutionResult(
  tx: Pick<TxRequest, 'from' | 'to' | 'functionData' | 'args'>,
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
    stateReads: [],
    stateTransitions: [],
  };
}
