import { PublicExecution, PublicExecutionResult, PublicExecutor } from '@aztec/acir-simulator';
import {
  ARGS_LENGTH,
  AztecAddress,
  CallContext,
  CircuitsWasm,
  CombinedAccumulatedData,
  EthAddress,
  Fr,
  FunctionData,
  GlobalVariables,
  HistoricBlockData,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  PUBLIC_DATA_TREE_HEIGHT,
  Proof,
  PublicCallRequest,
  PublicKernelPublicInputs,
  makeEmptyProof,
  makeTuple,
} from '@aztec/circuits.js';
import { computeCallStackItemHash } from '@aztec/circuits.js/abis';
import {
  makeAztecAddress,
  makePrivateKernelPublicInputsFinal,
  makePublicCallRequest,
  makeSelector,
} from '@aztec/circuits.js/factories';
import { padArrayEnd } from '@aztec/foundation/collection';
import { ExtendedContractData, FunctionCall, FunctionL2Logs, SiblingPath, Tx, TxL2Logs, mockTx } from '@aztec/types';
import { MerkleTreeOperations, TreeInfo } from '@aztec/world-state';

import { MockProxy, mock } from 'jest-mock-extended';
import times from 'lodash.times';

import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ContractsDataSourcePublicDB } from '../simulator/public_executor.js';
import { WasmPublicKernelCircuitSimulator } from '../simulator/public_kernel.js';
import { PublicProcessor } from './public_processor.js';

describe('public_processor', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let publicExecutor: MockProxy<PublicExecutor>;
  let publicProver: MockProxy<PublicProver>;
  let publicContractsDB: MockProxy<ContractsDataSourcePublicDB>;

  let proof: Proof;
  let root: Buffer;

  let processor: PublicProcessor;

  beforeEach(() => {
    db = mock<MerkleTreeOperations>();
    publicExecutor = mock<PublicExecutor>();
    publicProver = mock<PublicProver>();
    publicContractsDB = mock<ContractsDataSourcePublicDB>();

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
        HistoricBlockData.empty(),
        publicContractsDB,
      );
    });

    it('skips txs without public execution requests', async function () {
      const tx = mockTx();
      tx.data.end.publicCallStack = makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, Fr.zero);
      const hash = await tx.getTxHash();
      const [processed, failed] = await processor.process([tx]);

      expect(processed).toEqual([
        {
          isEmpty: false,
          hash,
          data: new PublicKernelPublicInputs(
            CombinedAccumulatedData.fromFinalAccumulatedData(tx.data.end),
            tx.data.constants,
          ),
          proof: tx.proof,
          encryptedLogs: tx.encryptedLogs,
          unencryptedLogs: tx.unencryptedLogs,
        },
      ]);
      expect(failed).toEqual([]);
    });

    it('returns failed txs without aborting entire operation', async function () {
      publicExecutor.simulate.mockRejectedValue(new Error(`Failed`));

      const tx = mockTx();
      const [processed, failed] = await processor.process([tx]);

      expect(processed).toEqual([]);
      expect(failed[0].tx).toEqual(tx);
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
      processor = new PublicProcessor(
        db,
        publicExecutor,
        publicKernel,
        publicProver,
        GlobalVariables.empty(),
        HistoricBlockData.empty(),
        publicContractsDB,
      );
    });

    const expectedTxByHash = async (tx: Tx) =>
      expect.objectContaining({
        hash: await tx.getTxHash(),
        proof,
      });

    it('runs a tx with enqueued public calls', async function () {
      const callRequests: PublicCallRequest[] = [makePublicCallRequest(0x100), makePublicCallRequest(0x100)];
      const callStackItems = await Promise.all(callRequests.map(call => call.toPublicCallStackItem()));
      const callStackHashes = callStackItems.map(call => computeCallStackItemHash(wasm, call));

      const kernelOutput = makePrivateKernelPublicInputsFinal(0x10);
      kernelOutput.end.publicCallStack = padArrayEnd(callStackHashes, Fr.ZERO, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX);
      kernelOutput.end.privateCallStack = padArrayEnd([], Fr.ZERO, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX);

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
      expect(processed).toEqual([await expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(2);
    });

    it('runs a tx with an enqueued public call with nested execution', async function () {
      const callRequest: PublicCallRequest = makePublicCallRequest(0x100);
      const callStackItem = await callRequest.toPublicCallStackItem();
      const callStackHash = computeCallStackItemHash(wasm, callStackItem);

      const kernelOutput = makePrivateKernelPublicInputsFinal(0x10);
      kernelOutput.end.publicCallStack = padArrayEnd([callStackHash], Fr.ZERO, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX);
      kernelOutput.end.privateCallStack = padArrayEnd([], Fr.ZERO, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX);

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
      expect(processed).toEqual([await expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(1);
    });
  });
});

function makePublicExecutionResultFromRequest(item: PublicCallRequest): PublicExecutionResult {
  return {
    execution: item,
    nestedExecutions: [],
    returnValues: [new Fr(1n)],
    newCommitments: [],
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
  const callContext = new CallContext(from, tx.to, EthAddress.ZERO, false, false, false);
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
    newCommitments: [],
    newNullifiers: [],
    newL2ToL1Messages: [],
    contractStorageReads: [],
    contractStorageUpdateRequests: [],
    unencryptedLogs: new FunctionL2Logs([]),
  };
}
