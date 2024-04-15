import {
  type BlockProver,
  EncryptedTxL2Logs,
  type ProcessedTx,
  PublicDataWrite,
  SiblingPath,
  SimulationError,
  Tx,
  type TxValidator,
  UnencryptedTxL2Logs,
  mockTx,
  toTxEffect,
} from '@aztec/circuit-types';
import {
  ContractStorageUpdateRequest,
  Fr,
  GlobalVariables,
  Header,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  PUBLIC_DATA_TREE_HEIGHT,
  type Proof,
  type PublicCallRequest,
  PublicDataUpdateRequest,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import {
  fr,
  makeAztecAddress,
  makePrivateKernelTailCircuitPublicInputs,
  makePublicCallRequest,
  makeSelector,
} from '@aztec/circuits.js/testing';
import { makeTuple } from '@aztec/foundation/array';
import { arrayNonEmptyLength, times } from '@aztec/foundation/collection';
import { type PublicExecutionResult, type PublicExecutor, WASMSimulator } from '@aztec/simulator';
import { type MerkleTreeOperations, type TreeInfo } from '@aztec/world-state';

import { jest } from '@jest/globals';
import { type MockProxy, mock } from 'jest-mock-extended';

import { PublicExecutionResultBuilder, addKernelPublicCallStack, makeFunctionCall } from '../mocks/fixtures.js';
import { type ContractsDataSourcePublicDB, type WorldStatePublicDB } from './public_executor.js';
import { RealPublicKernelCircuitSimulator } from './public_kernel.js';
import { type PublicKernelCircuitSimulator } from './public_kernel_circuit_simulator.js';
import { PublicProcessor } from './public_processor.js';

describe('public_processor', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let publicExecutor: MockProxy<PublicExecutor>;
  let publicContractsDB: MockProxy<ContractsDataSourcePublicDB>;
  let publicWorldStateDB: MockProxy<WorldStatePublicDB>;
  let prover: MockProxy<BlockProver>;

  let proof: Proof;
  let root: Buffer;

  let processor: PublicProcessor;

  beforeEach(() => {
    db = mock<MerkleTreeOperations>();
    publicExecutor = mock<PublicExecutor>();
    publicContractsDB = mock<ContractsDataSourcePublicDB>();
    publicWorldStateDB = mock<WorldStatePublicDB>();
    prover = mock<BlockProver>();

    proof = makeEmptyProof();
    root = Buffer.alloc(32, 5);

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
        GlobalVariables.empty(),
        Header.empty(),
        publicContractsDB,
        publicWorldStateDB,
      );
    });

    it('skips txs without public execution requests', async function () {
      const tx = mockTx(1, {
        numberOfNonRevertiblePublicCallRequests: 0,
        numberOfRevertiblePublicCallRequests: 0,
      });

      const hash = tx.getTxHash();
      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed.length).toBe(1);

      const p = processed[0];
      const e: ProcessedTx = {
        hash,
        data: tx.data.toKernelCircuitPublicInputs(),
        proof: tx.proof,
        encryptedLogs: tx.encryptedLogs,
        unencryptedLogs: tx.unencryptedLogs,
        isEmpty: false,
        revertReason: undefined,
        publicKernelRequests: [],
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

      expect(prover.addNewTx).toHaveBeenCalledWith(p);
    });

    it('returns failed txs without aborting entire operation', async function () {
      publicExecutor.simulate.mockRejectedValue(new SimulationError(`Failed`, []));

      const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 1 });
      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed).toEqual([]);
      expect(failed[0].tx).toEqual(tx);
      expect(failed[0].error).toEqual(new SimulationError(`Failed`, []));
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(1);
      expect(prover.addNewTx).toHaveBeenCalledTimes(0);
    });
  });

  describe('with actual circuits', () => {
    let publicKernel: PublicKernelCircuitSimulator;

    beforeEach(() => {
      const path = times(PUBLIC_DATA_TREE_HEIGHT, i => Buffer.alloc(32, i));
      db.getSiblingPath.mockResolvedValue(new SiblingPath<number>(PUBLIC_DATA_TREE_HEIGHT, path));
      publicKernel = new RealPublicKernelCircuitSimulator(new WASMSimulator());
      processor = new PublicProcessor(
        db,
        publicExecutor,
        publicKernel,
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
      const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 2 });

      publicExecutor.simulate.mockImplementation(execution => {
        for (const request of tx.enqueuedPublicFunctionCalls) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            const result = PublicExecutionResultBuilder.fromPublicCallRequest({ request }).build();
            // result.unencryptedLogs = tx.unencryptedLogs.functionLogs[0];
            return Promise.resolve(result);
          }
        }
        throw new Error(`Unexpected execution request: ${execution}`);
      });

      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(2);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(0);

      expect(prover.addNewTx).toHaveBeenCalledWith(processed[0]);
    });

    it('runs a tx with an enqueued public call with nested execution', async function () {
      const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 1 });
      const callRequest = tx.enqueuedPublicFunctionCalls[0];

      const publicExecutionResult = PublicExecutionResultBuilder.fromPublicCallRequest({
        request: callRequest,
        nestedExecutions: [
          PublicExecutionResultBuilder.fromFunctionCall({
            from: callRequest.contractAddress,
            tx: makeFunctionCall(),
          }).build(),
        ],
      }).build();

      publicExecutor.simulate.mockResolvedValue(publicExecutionResult);

      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.checkpoint).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollbackToCheckpoint).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(0);

      expect(prover.addNewTx).toHaveBeenCalledWith(processed[0]);
    });

    it('does not attempt to overfill a block', async function () {
      const txs = Array.from([1, 2, 3], index =>
        mockTx(index, { numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 1 }),
      );

      let txCount = 0;

      publicExecutor.simulate.mockImplementation(execution => {
        const tx = txs[txCount++];
        for (const request of tx.enqueuedPublicFunctionCalls) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            const result = PublicExecutionResultBuilder.fromPublicCallRequest({ request }).build();
            // result.unencryptedLogs = tx.unencryptedLogs.functionLogs[0];
            return Promise.resolve(result);
          }
        }
        throw new Error(`Unexpected execution request: ${execution}`);
      });

      // We are passing 3 txs but only 2 can fit in the block
      const [processed, failed] = await processor.process(txs, 2, prover);

      expect(processed).toHaveLength(2);
      expect(processed).toEqual([expectedTxByHash(txs[0]), expectedTxByHash(txs[1])]);
      expect(failed).toHaveLength(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(2);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(2);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(0);

      expect(prover.addNewTx).toHaveBeenCalledWith(processed[0]);
      expect(prover.addNewTx).toHaveBeenCalledWith(processed[1]);
    });

    it('does not send a transaction to the prover if validation fails', async function () {
      const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 1 });

      publicExecutor.simulate.mockImplementation(execution => {
        for (const request of tx.enqueuedPublicFunctionCalls) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            const result = PublicExecutionResultBuilder.fromPublicCallRequest({ request }).build();
            // result.unencryptedLogs = tx.unencryptedLogs.functionLogs[0];
            return Promise.resolve(result);
          }
        }
        throw new Error(`Unexpected execution request: ${execution}`);
      });

      const txValidator: MockProxy<TxValidator<ProcessedTx>> = mock();
      txValidator.validateTxs.mockRejectedValue([[], [tx]]);

      const [processed, failed] = await processor.process([tx], 1, prover, txValidator);

      expect(processed).toHaveLength(0);
      expect(failed).toHaveLength(1);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(1);

      expect(prover.addNewTx).toHaveBeenCalledTimes(0);
    });

    it('rolls back app logic db updates on failed public execution, but persists setup/teardown', async function () {
      const baseContractAddressSeed = 0x200;
      const baseContractAddress = makeAztecAddress(baseContractAddressSeed);
      const callRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      callRequests[0].callContext.sideEffectCounter = 2;
      callRequests[1].callContext.sideEffectCounter = 3;
      callRequests[2].callContext.sideEffectCounter = 4;

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);
      kernelOutput.forPublic!.endNonRevertibleData.publicDataUpdateRequests = makeTuple(
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        PublicDataUpdateRequest.empty,
      );
      kernelOutput.forPublic!.end.publicDataUpdateRequests = makeTuple(
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        PublicDataUpdateRequest.empty,
      );

      addKernelPublicCallStack(kernelOutput, {
        setupCalls: [callRequests[0]],
        appLogicCalls: [callRequests[2]],
        teardownCall: callRequests[1],
      });

      const tx = new Tx(
        kernelOutput,
        proof,
        EncryptedTxL2Logs.empty(),
        UnencryptedTxL2Logs.empty(),
        // reverse because `enqueuedPublicFunctions` expects the last element to be the front of the queue
        callRequests.slice().reverse(),
      );

      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;
      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[0],
          contractStorageUpdateRequests: [new ContractStorageUpdateRequest(contractSlotA, fr(0x101))],
        }).build(),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[2],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x102)),
                new ContractStorageUpdateRequest(contractSlotB, fr(0x151)),
              ],
            }).build(),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              revertReason: new SimulationError('Simulation Failed', []),
            }).build(),
          ],
        }).build(),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [new ContractStorageUpdateRequest(contractSlotC, fr(0x201))],
            }).build(),
          ],
        }).build(),
      ];

      publicExecutor.simulate.mockImplementation(execution => {
        if (simulatorCallCount < simulatorResults.length) {
          return Promise.resolve(simulatorResults[simulatorCallCount++]);
        } else {
          throw new Error(`Unexpected execution request: ${execution}, call count: ${simulatorCallCount}`);
        }
      });

      const setupSpy = jest.spyOn(publicKernel, 'publicKernelCircuitSetup');
      const appLogicSpy = jest.spyOn(publicKernel, 'publicKernelCircuitAppLogic');
      const teardownSpy = jest.spyOn(publicKernel, 'publicKernelCircuitTeardown');

      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);

      expect(setupSpy).toHaveBeenCalledTimes(1);
      expect(appLogicSpy).toHaveBeenCalledTimes(2);
      expect(teardownSpy).toHaveBeenCalledTimes(2);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(3);
      expect(publicWorldStateDB.checkpoint).toHaveBeenCalledTimes(2);
      expect(publicWorldStateDB.rollbackToCheckpoint).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(0);

      const txEffect = toTxEffect(processed[0]);
      expect(arrayNonEmptyLength(txEffect.publicDataWrites, PublicDataWrite.isEmpty)).toEqual(2);
      expect(txEffect.publicDataWrites[0]).toEqual(
        new PublicDataWrite(computePublicDataTreeLeafSlot(baseContractAddress, contractSlotA), fr(0x101)),
      );
      expect(txEffect.publicDataWrites[1]).toEqual(
        new PublicDataWrite(computePublicDataTreeLeafSlot(baseContractAddress, contractSlotC), fr(0x201)),
      );
      expect(txEffect.encryptedLogs.getTotalLogCount()).toBe(0);
      expect(txEffect.unencryptedLogs.getTotalLogCount()).toBe(0);

      expect(prover.addNewTx).toHaveBeenCalledWith(processed[0]);
    });

    it('fails a transaction that reverts in setup', async function () {
      const baseContractAddressSeed = 0x200;
      const baseContractAddress = makeAztecAddress(baseContractAddressSeed);
      const callRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      callRequests[0].callContext.sideEffectCounter = 2;
      callRequests[1].callContext.sideEffectCounter = 3;
      callRequests[2].callContext.sideEffectCounter = 4;

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);

      addKernelPublicCallStack(kernelOutput, {
        setupCalls: [callRequests[0]],
        appLogicCalls: [callRequests[2]],
        teardownCall: callRequests[1],
      });

      const tx = new Tx(
        kernelOutput,
        proof,
        EncryptedTxL2Logs.empty(),
        UnencryptedTxL2Logs.empty(),
        // reverse because `enqueuedPublicFunctions` expects the last element to be the front of the queue
        callRequests.slice().reverse(),
      );

      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;
      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[0],
          contractStorageUpdateRequests: [new ContractStorageUpdateRequest(contractSlotA, fr(0x101))],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x102)),
                new ContractStorageUpdateRequest(contractSlotB, fr(0x151)),
              ],
            }).build(),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              revertReason: new SimulationError('Simulation Failed', []),
            }).build(),
          ],
        }).build(),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[2],
        }).build(),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [new ContractStorageUpdateRequest(contractSlotC, fr(0x201))],
            }).build(),
          ],
        }).build(),
      ];

      publicExecutor.simulate.mockImplementation(execution => {
        if (simulatorCallCount < simulatorResults.length) {
          return Promise.resolve(simulatorResults[simulatorCallCount++]);
        } else {
          throw new Error(`Unexpected execution request: ${execution}, call count: ${simulatorCallCount}`);
        }
      });

      const setupSpy = jest.spyOn(publicKernel, 'publicKernelCircuitSetup');
      const appLogicSpy = jest.spyOn(publicKernel, 'publicKernelCircuitAppLogic');
      const teardownSpy = jest.spyOn(publicKernel, 'publicKernelCircuitTeardown');

      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed).toHaveLength(0);
      expect(failed).toHaveLength(1);
      expect(failed[0].tx.getTxHash()).toEqual(tx.getTxHash());

      expect(setupSpy).toHaveBeenCalledTimes(1);
      expect(appLogicSpy).toHaveBeenCalledTimes(0);
      expect(teardownSpy).toHaveBeenCalledTimes(0);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(1);

      expect(publicWorldStateDB.checkpoint).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.rollbackToCheckpoint).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(1);

      expect(prover.addNewTx).toHaveBeenCalledTimes(0);
    });

    it('fails a transaction that reverts in teardown', async function () {
      const baseContractAddressSeed = 0x200;
      const baseContractAddress = makeAztecAddress(baseContractAddressSeed);
      const callRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      callRequests[0].callContext.sideEffectCounter = 2;
      callRequests[1].callContext.sideEffectCounter = 3;
      callRequests[2].callContext.sideEffectCounter = 4;

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);

      addKernelPublicCallStack(kernelOutput, {
        setupCalls: [callRequests[0]],
        appLogicCalls: [callRequests[2]],
        teardownCall: callRequests[1],
      });

      const tx = new Tx(
        kernelOutput,
        proof,
        EncryptedTxL2Logs.empty(),
        UnencryptedTxL2Logs.empty(),
        // reverse because `enqueuedPublicFunctions` expects the last element to be the front of the queue
        callRequests.slice().reverse(),
      );

      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;
      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[0],
          contractStorageUpdateRequests: [new ContractStorageUpdateRequest(contractSlotA, fr(0x101))],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x102)),
                new ContractStorageUpdateRequest(contractSlotB, fr(0x151)),
              ],
            }).build(),
          ],
        }).build(),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[2],
        }).build(),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              revertReason: new SimulationError('Simulation Failed', []),
            }).build(),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [new ContractStorageUpdateRequest(contractSlotC, fr(0x201))],
            }).build(),
          ],
        }).build(),
      ];

      publicExecutor.simulate.mockImplementation(execution => {
        if (simulatorCallCount < simulatorResults.length) {
          return Promise.resolve(simulatorResults[simulatorCallCount++]);
        } else {
          throw new Error(`Unexpected execution request: ${execution}, call count: ${simulatorCallCount}`);
        }
      });

      const setupSpy = jest.spyOn(publicKernel, 'publicKernelCircuitSetup');
      const appLogicSpy = jest.spyOn(publicKernel, 'publicKernelCircuitAppLogic');
      const teardownSpy = jest.spyOn(publicKernel, 'publicKernelCircuitTeardown');

      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed).toHaveLength(0);
      expect(failed).toHaveLength(1);
      expect(failed[0].tx.getTxHash()).toEqual(tx.getTxHash());

      expect(setupSpy).toHaveBeenCalledTimes(2);
      expect(appLogicSpy).toHaveBeenCalledTimes(1);
      expect(teardownSpy).toHaveBeenCalledTimes(2);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(3);
      expect(publicWorldStateDB.checkpoint).toHaveBeenCalledTimes(2);
      expect(publicWorldStateDB.rollbackToCheckpoint).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(1);

      expect(prover.addNewTx).toHaveBeenCalledTimes(0);
    });

    it('runs a tx with setup and teardown phases', async function () {
      const baseContractAddressSeed = 0x200;
      const baseContractAddress = makeAztecAddress(baseContractAddressSeed);
      const callRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      callRequests[0].callContext.sideEffectCounter = 2;
      callRequests[1].callContext.sideEffectCounter = 3;
      callRequests[2].callContext.sideEffectCounter = 4;

      const kernelOutput = makePrivateKernelTailCircuitPublicInputs(0x10);
      kernelOutput.forPublic!.end.encryptedLogsHash = Fr.ZERO;
      kernelOutput.forPublic!.end.unencryptedLogsHash = Fr.ZERO;
      kernelOutput.forPublic!.endNonRevertibleData.publicDataUpdateRequests = makeTuple(
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        PublicDataUpdateRequest.empty,
      );
      kernelOutput.forPublic!.end.publicDataUpdateRequests = makeTuple(
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        PublicDataUpdateRequest.empty,
      );

      addKernelPublicCallStack(kernelOutput, {
        setupCalls: [callRequests[0]],
        appLogicCalls: [callRequests[2]],
        teardownCall: callRequests[1],
      });

      const tx = new Tx(
        kernelOutput,
        proof,
        EncryptedTxL2Logs.empty(),
        UnencryptedTxL2Logs.empty(),
        // reverse because `enqueuedPublicFunctions` expects the last element to be the front of the queue
        callRequests.slice().reverse(),
      );

      // const baseContractAddress = makeAztecAddress(30);
      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;
      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({ request: callRequests[0] }).build(),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[2],
          contractStorageUpdateRequests: [
            new ContractStorageUpdateRequest(contractSlotA, fr(0x101)),
            new ContractStorageUpdateRequest(contractSlotB, fr(0x151)),
          ],
        }).build(),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: callRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x101)),
                new ContractStorageUpdateRequest(contractSlotC, fr(0x201)),
              ],
            }).build(),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: callRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [new ContractStorageUpdateRequest(contractSlotA, fr(0x102))],
            }).build(),
          ],
        }).build(),
      ];

      publicExecutor.simulate.mockImplementation(execution => {
        if (simulatorCallCount < simulatorResults.length) {
          return Promise.resolve(simulatorResults[simulatorCallCount++]);
        } else {
          throw new Error(`Unexpected execution request: ${execution}, call count: ${simulatorCallCount}`);
        }
      });

      const setupSpy = jest.spyOn(publicKernel, 'publicKernelCircuitSetup');
      const appLogicSpy = jest.spyOn(publicKernel, 'publicKernelCircuitAppLogic');
      const teardownSpy = jest.spyOn(publicKernel, 'publicKernelCircuitTeardown');

      const [processed, failed] = await processor.process([tx], 1, prover);

      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(failed).toHaveLength(0);

      expect(setupSpy).toHaveBeenCalledTimes(1);
      expect(appLogicSpy).toHaveBeenCalledTimes(1);
      expect(teardownSpy).toHaveBeenCalledTimes(3);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(3);
      expect(publicWorldStateDB.checkpoint).toHaveBeenCalledTimes(3);
      expect(publicWorldStateDB.rollbackToCheckpoint).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(0);

      const txEffect = toTxEffect(processed[0]);
      expect(arrayNonEmptyLength(txEffect.publicDataWrites, PublicDataWrite.isEmpty)).toEqual(3);
      expect(txEffect.publicDataWrites[0]).toEqual(
        new PublicDataWrite(computePublicDataTreeLeafSlot(baseContractAddress, contractSlotC), fr(0x201)),
      );
      expect(txEffect.publicDataWrites[1]).toEqual(
        new PublicDataWrite(computePublicDataTreeLeafSlot(baseContractAddress, contractSlotA), fr(0x102)),
      );
      expect(txEffect.publicDataWrites[2]).toEqual(
        new PublicDataWrite(computePublicDataTreeLeafSlot(baseContractAddress, contractSlotB), fr(0x151)),
      );
      expect(txEffect.encryptedLogs.getTotalLogCount()).toBe(0);
      expect(txEffect.unencryptedLogs.getTotalLogCount()).toBe(0);

      expect(prover.addNewTx).toHaveBeenCalledWith(processed[0]);
    });
  });
});
