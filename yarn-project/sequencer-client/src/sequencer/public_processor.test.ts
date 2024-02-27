import {
  ExtendedContractData,
  FunctionCall,
  FunctionL2Logs,
  SiblingPath,
  SimulationError,
  Tx,
  TxL2Logs,
  mockTx,
} from '@aztec/circuit-types';
import {
  ARGS_LENGTH,
  AztecAddress,
  CallContext,
  CallRequest,
  EthAddress,
  Fr,
  FunctionData,
  GlobalVariables,
  Header,
  MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  PUBLIC_DATA_TREE_HEIGHT,
  Proof,
  PublicAccumulatedNonRevertibleData,
  PublicAccumulatedRevertibleData,
  PublicCallRequest,
  PublicKernelCircuitPublicInputs,
  makeEmptyProof,
} from '@aztec/circuits.js';
import {
  makeAztecAddress,
  makePrivateKernelTailCircuitPublicInputs,
  makePublicCallRequest,
  makeSelector,
} from '@aztec/circuits.js/testing';
import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd, times } from '@aztec/foundation/collection';
import { PublicExecution, PublicExecutionResult, PublicExecutor } from '@aztec/simulator';
import { MerkleTreeOperations, TreeInfo } from '@aztec/world-state';

import { MockProxy, mock } from 'jest-mock-extended';

import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ContractsDataSourcePublicDB, WorldStatePublicDB } from '../simulator/public_executor.js';
import { RealPublicKernelCircuitSimulator } from '../simulator/public_kernel.js';
import { ProcessedTx } from './processed_tx.js';
import { PublicProcessor } from './public_processor.js';

describe('public_processor', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let publicExecutor: MockProxy<PublicExecutor>;
  let publicProver: MockProxy<PublicProver>;
  let publicContractsDB: MockProxy<ContractsDataSourcePublicDB>;
  let publicWorldStateDB: MockProxy<WorldStatePublicDB>;

  let proof: Proof;
  let root: Buffer;

  let processor: PublicProcessor;

  beforeEach(() => {
    db = mock<MerkleTreeOperations>();
    publicExecutor = mock<PublicExecutor>();
    publicProver = mock<PublicProver>();
    publicContractsDB = mock<ContractsDataSourcePublicDB>();
    publicWorldStateDB = mock<WorldStatePublicDB>();

    proof = makeEmptyProof();
    root = Buffer.alloc(32, 5);

    publicProver.getPublicCircuitProof.mockResolvedValue(proof);
    publicProver.getPublicKernelCircuitProof.mockResolvedValue(proof);
    db.getTreeInfo.mockResolvedValue({ root } as TreeInfo);
  });

  describe('with mock circuits', () => {
    let publicKernel: MockProxy<PublicKernelCircuitSimulator>;

    beforeEach(() => {
      publicKernel = mock<PublicKernelCircuitSimulator>();
      processor = new PublicProcessor(
        db,
        publicExecutor,
        publicKernel,
        publicProver,
        GlobalVariables.empty(),
        Header.empty(),
        publicContractsDB,
        publicWorldStateDB,
      );
    });

    it('skips txs without public execution requests', async function () {
      const tx = mockTx();
      tx.data.end.publicCallStack = makeTuple(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty);
      tx.data.endNonRevertibleData.publicCallStack = makeTuple(
        MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
        CallRequest.empty,
      );
      tx.data.needsSetup = false;
      tx.data.needsAppLogic = false;
      tx.data.needsTeardown = false;

      const hash = tx.getTxHash();
      const [processed, failed] = await processor.process([tx]);

      expect(processed.length).toBe(1);

      const p = processed[0];
      const e = {
        hash,
        data: new PublicKernelCircuitPublicInputs(
          tx.data.aggregationObject,
          PublicAccumulatedNonRevertibleData.fromPrivateAccumulatedNonRevertibleData(tx.data.endNonRevertibleData),
          PublicAccumulatedRevertibleData.fromPrivateAccumulatedRevertibleData(tx.data.end),
          tx.data.constants,
          tx.data.needsSetup,
          tx.data.needsAppLogic,
          tx.data.needsTeardown,
        ),
        proof: tx.proof,
        encryptedLogs: tx.encryptedLogs,
        unencryptedLogs: tx.unencryptedLogs,
        newContracts: tx.newContracts,
        isEmpty: false,
      };

      // Jest is complaining that the two objects are not equal, but they are.
      // It expects something and says "Received: serializes to the same string"
      // TODO why can't we just expect(p).toEqual(e) here anymore?
      expect(Object.keys(p)).toEqual(Object.keys(e));
      for (const key in e) {
        if (key === 'data') {
          expect(p.data.toBuffer()).toEqual(e.data.toBuffer());
        } else {
          expect(p[key as keyof ProcessedTx]).toEqual(e[key as keyof ProcessedTx]);
        }
      }

      expect(failed).toEqual([]);
    });

    it('returns failed txs without aborting entire operation', async function () {
      publicExecutor.simulate.mockRejectedValue(new Error(`Failed`));

      const tx = mockTx();
      const [processed, failed] = await processor.process([tx]);

      expect(processed).toEqual([]);
      expect(failed[0].tx).toEqual(tx);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollback).toHaveBeenCalledTimes(1);
    });
  });

  describe('with actual circuits', () => {
    let publicKernel: PublicKernelCircuitSimulator;

    beforeEach(() => {
      const path = times(PUBLIC_DATA_TREE_HEIGHT, i => Buffer.alloc(32, i));
      db.getSiblingPath.mockResolvedValue(new SiblingPath<number>(PUBLIC_DATA_TREE_HEIGHT, path));
      publicKernel = new RealPublicKernelCircuitSimulator();
      processor = new PublicProcessor(
        db,
        publicExecutor,
        publicKernel,
        publicProver,
        GlobalVariables.empty(),
        Header.empty(),
        publicContractsDB,
        publicWorldStateDB,
      );
    });

    const expectedTxByHash = (tx: Tx) =>
      expect.objectContaining({
        hash: tx.getTxHash(),
        proof,
      });

    it('runs a tx with enqueued public calls', async function () {
      const publicCallRequests: PublicCallRequest[] = [makePublicCallRequest(0x100), makePublicCallRequest(0x100)];
      const callRequests = publicCallRequests.map(call => call.toCallRequest());

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);
      kernelOutput.end.publicCallStack = padArrayEnd(
        callRequests,
        CallRequest.empty(),
        MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      );
      kernelOutput.end.privateCallStack = padArrayEnd([], CallRequest.empty(), MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX);

      kernelOutput.endNonRevertibleData.publicCallStack = makeTuple(
        MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
        CallRequest.empty,
      );

      const tx = new Tx(kernelOutput, proof, TxL2Logs.random(2, 3), TxL2Logs.random(3, 2), publicCallRequests, [
        ExtendedContractData.random(),
      ]);

      tx.data.needsSetup = false;
      tx.data.needsTeardown = false;

      publicExecutor.simulate.mockImplementation(execution => {
        for (const request of publicCallRequests) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            return Promise.resolve(makePublicExecutionResultFromRequest(request));
          }
        }
        throw new Error(`Unexpected execution request: ${execution}`);
      });

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(2);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollback).toHaveBeenCalledTimes(0);
    });

    it('runs a tx with an enqueued public call with nested execution', async function () {
      const callRequest: PublicCallRequest = makePublicCallRequest(0x100);
      const callStackItem = callRequest.toCallRequest();

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);
      kernelOutput.end.publicCallStack = padArrayEnd(
        [callStackItem],
        CallRequest.empty(),
        MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      );
      kernelOutput.end.privateCallStack = padArrayEnd([], CallRequest.empty(), MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX);
      kernelOutput.endNonRevertibleData.publicCallStack = makeTuple(
        MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
        CallRequest.empty,
      );

      kernelOutput.needsSetup = false;
      kernelOutput.needsTeardown = false;

      const tx = new Tx(
        kernelOutput,
        proof,
        TxL2Logs.random(2, 3),
        TxL2Logs.random(3, 2),
        [callRequest],
        [ExtendedContractData.random()],
      );

      const publicExecutionResult = makePublicExecutionResultFromRequest(callRequest);
      publicExecutionResult.nestedExecutions = [
        makePublicExecutionResult(publicExecutionResult.execution.contractAddress, {
          to: makeAztecAddress(30),
          functionData: new FunctionData(makeSelector(5), false, false, false),
          args: new Array(ARGS_LENGTH).fill(Fr.ZERO),
        }),
      ];
      publicExecutor.simulate.mockResolvedValue(publicExecutionResult);

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollback).toHaveBeenCalledTimes(0);
    });

    it('rolls back db updates on failed public execution', async function () {
      const callRequest: PublicCallRequest = makePublicCallRequest(0x100);
      const callStackItem = callRequest.toCallRequest();

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);
      kernelOutput.end.publicCallStack = padArrayEnd(
        [callStackItem],
        CallRequest.empty(),
        MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      );
      kernelOutput.end.privateCallStack = padArrayEnd([], CallRequest.empty(), MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX);
      kernelOutput.endNonRevertibleData.publicCallStack = makeTuple(
        MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
        CallRequest.empty,
      );
      const tx = new Tx(
        kernelOutput,
        proof,
        TxL2Logs.random(2, 3),
        TxL2Logs.random(3, 2),
        [callRequest],
        [ExtendedContractData.random()],
      );

      tx.data.needsSetup = false;
      tx.data.needsTeardown = false;

      const publicExecutionResult = makePublicExecutionResultFromRequest(callRequest);
      publicExecutionResult.nestedExecutions = [
        makePublicExecutionResult(publicExecutionResult.execution.contractAddress, {
          to: makeAztecAddress(30),
          functionData: new FunctionData(makeSelector(5), false, false, false),
          args: new Array(ARGS_LENGTH).fill(Fr.ZERO),
        }),
      ];
      publicExecutor.simulate.mockRejectedValueOnce(new SimulationError('Simulation Failed', []));

      const [processed, failed] = await processor.process([tx]);

      expect(failed).toHaveLength(1);
      expect(processed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollback).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(0);
    });

    it('runs a tx with setup and teardown phases', async function () {
      const callRequests: PublicCallRequest[] = [0x100, 0x200, 0x300].map(makePublicCallRequest);
      callRequests[0].callContext.startSideEffectCounter = 2;
      callRequests[1].callContext.startSideEffectCounter = 3;
      callRequests[2].callContext.startSideEffectCounter = 4;

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);

      // the first two calls are non-revertible
      // the first is for setup, the second is for teardown
      kernelOutput.endNonRevertibleData.publicCallStack = padArrayEnd(
        // this is a stack, so the first item is the last call
        // and callRequests is in the order of the calls
        [callRequests[1].toCallRequest(), callRequests[0].toCallRequest()],
        CallRequest.empty(),
        MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      );

      kernelOutput.end.publicCallStack = padArrayEnd(
        [callRequests[2].toCallRequest()],
        CallRequest.empty(),
        MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      );
      kernelOutput.end.privateCallStack = padArrayEnd([], CallRequest.empty(), MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX);
      callRequests.reverse();

      const tx = new Tx(kernelOutput, proof, TxL2Logs.random(2, 3), TxL2Logs.random(3, 2), callRequests, [
        ExtendedContractData.random(),
      ]);

      publicExecutor.simulate.mockImplementation(execution => {
        for (const request of callRequests) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            return Promise.resolve(makePublicExecutionResultFromRequest(request));
          }
        }
        throw new Error(`Unexpected execution request: ${execution}`);
      });

      const [processed, failed] = await processor.process([tx]);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(3);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(3);
      expect(publicWorldStateDB.rollback).toHaveBeenCalledTimes(0);
    });
  });
});

function makePublicExecutionResultFromRequest(item: PublicCallRequest): PublicExecutionResult {
  return {
    execution: item,
    nestedExecutions: [],
    returnValues: [new Fr(1n)],
    newNoteHashes: [],
    newL2ToL1Messages: [],
    newNullifiers: [],
    contractStorageReads: [],
    contractStorageUpdateRequests: [],
    unencryptedLogs: new FunctionL2Logs([]),
  };
}

function makePublicExecutionResult(
  from: AztecAddress,
  tx: FunctionCall,
  nestedExecutions: PublicExecutionResult[] = [],
): PublicExecutionResult {
  const callContext = new CallContext(from, tx.to, EthAddress.ZERO, tx.functionData.selector, false, false, false, 0);
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
    newNoteHashes: [],
    newNullifiers: [],
    newL2ToL1Messages: [],
    contractStorageReads: [],
    contractStorageUpdateRequests: [],
    unencryptedLogs: new FunctionL2Logs([]),
  };
}
