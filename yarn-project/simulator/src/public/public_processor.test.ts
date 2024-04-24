import {
  type BlockProver,
  type ProcessedTx,
  PublicDataWrite,
  SimulationError,
  type Tx,
  type TxValidator,
  mockTx,
  toTxEffect,
} from '@aztec/circuit-types';
import {
  AppendOnlyTreeSnapshot,
  ContractStorageUpdateRequest,
  Fr,
  Gas,
  GasFees,
  GasSettings,
  GlobalVariables,
  Header,
  PUBLIC_DATA_TREE_HEIGHT,
  PartialStateReference,
  type Proof,
  type PublicCallRequest,
  PublicDataTreeLeafPreimage,
  StateReference,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import { fr, makeAztecAddress, makePublicCallRequest, makeSelector } from '@aztec/circuits.js/testing';
import { arrayNonEmptyLength } from '@aztec/foundation/collection';
import { type FieldsOf } from '@aztec/foundation/types';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type AppendOnlyTree, Pedersen, StandardTree, newTree } from '@aztec/merkle-tree';
import { type PublicExecutionResult, type PublicExecutor, WASMSimulator } from '@aztec/simulator';
import { type MerkleTreeOperations, type TreeInfo } from '@aztec/world-state';

import { jest } from '@jest/globals';
import { type MockProxy, mock } from 'jest-mock-extended';

import { PublicExecutionResultBuilder, makeFunctionCall } from '../mocks/fixtures.js';
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
    let publicDataTree: AppendOnlyTree<Fr>;

    const mockTxWithPartialState = (
      {
        hasLogs = false,
        numberOfNonRevertiblePublicCallRequests = 0,
        numberOfRevertiblePublicCallRequests = 0,
        publicCallRequests = [],
      }: {
        hasLogs?: boolean;
        numberOfNonRevertiblePublicCallRequests?: number;
        numberOfRevertiblePublicCallRequests?: number;
        publicCallRequests?: PublicCallRequest[];
      } = {},
      seed = 1,
    ) => {
      return mockTx(seed, {
        hasLogs,
        numberOfNonRevertiblePublicCallRequests,
        numberOfRevertiblePublicCallRequests,
        publicCallRequests,
      });
    };

    beforeAll(async () => {
      publicDataTree = await newTree(
        StandardTree,
        openTmpStore(),
        new Pedersen(),
        'PublicData',
        Fr,
        PUBLIC_DATA_TREE_HEIGHT,
        1, // Add a default low leaf for the public data hints to be proved against.
      );
    });

    beforeEach(() => {
      const snap = new AppendOnlyTreeSnapshot(
        Fr.fromBuffer(publicDataTree.getRoot(true)),
        Number(publicDataTree.getNumLeaves(true)),
      );

      const header = Header.empty();
      const stateReference = new StateReference(
        header.state.l1ToL2MessageTree,
        new PartialStateReference(header.state.partial.noteHashTree, header.state.partial.nullifierTree, snap),
      );
      // Clone the whole state because somewhere down the line (AbstractPhaseManager) the public data root is modified in the referenced header directly :/
      header.state = StateReference.fromBuffer(stateReference.toBuffer());

      db.getStateReference.mockResolvedValue(stateReference);
      db.getSiblingPath.mockResolvedValue(publicDataTree.getSiblingPath(0n, false));
      db.getPreviousValueIndex.mockResolvedValue({ index: 0n, alreadyPresent: true });
      db.getLeafPreimage.mockResolvedValue(new PublicDataTreeLeafPreimage(new Fr(0), new Fr(0), new Fr(0), 0n));

      publicKernel = new RealPublicKernelCircuitSimulator(new WASMSimulator());
      processor = new PublicProcessor(
        db,
        publicExecutor,
        publicKernel,
        GlobalVariables.from({ ...GlobalVariables.empty(), gasFees: GasFees.default() }),
        header,
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
      const tx = mockTxWithPartialState({
        numberOfRevertiblePublicCallRequests: 2,
      });

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

      expect(failed.map(f => f.error)).toEqual([]);
      expect(processed).toHaveLength(1);
      expect(processed).toEqual([expectedTxByHash(tx)]);
      expect(publicExecutor.simulate).toHaveBeenCalledTimes(2);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(0);

      expect(prover.addNewTx).toHaveBeenCalledWith(processed[0]);
    });

    it('runs a tx with an enqueued public call with nested execution', async function () {
      const tx = mockTxWithPartialState({ numberOfRevertiblePublicCallRequests: 1 });
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
        mockTxWithPartialState({ numberOfRevertiblePublicCallRequests: 1 }, index),
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
      const tx = mockTxWithPartialState({ numberOfRevertiblePublicCallRequests: 1 });

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
      const publicCallRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      publicCallRequests[0].callContext.sideEffectCounter = 2;
      publicCallRequests[1].callContext.sideEffectCounter = 3;
      publicCallRequests[2].callContext.sideEffectCounter = 4;

      const tx = mockTxWithPartialState({
        numberOfNonRevertiblePublicCallRequests: 2,
        numberOfRevertiblePublicCallRequests: 1,
        publicCallRequests,
      });

      const teardownGas = tx.data.constants.txContext.gasSettings.getTeardownLimits();
      const teardownResultSettings = { startGasLeft: teardownGas, endGasLeft: teardownGas };

      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;
      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[0],
          contractStorageUpdateRequests: [
            new ContractStorageUpdateRequest(contractSlotA, fr(0x101), 11, baseContractAddress),
          ],
        }).build(),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[2],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x102), 13, baseContractAddress),
                new ContractStorageUpdateRequest(contractSlotB, fr(0x151), 14, baseContractAddress),
              ],
            }).build(),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              revertReason: new SimulationError('Simulation Failed', []),
            }).build(),
          ],
        }).build(),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotC, fr(0x201), 12, baseContractAddress),
              ],
            }).build(teardownResultSettings),
          ],
        }).build(teardownResultSettings),
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
      const publicCallRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      publicCallRequests[0].callContext.sideEffectCounter = 2;
      publicCallRequests[1].callContext.sideEffectCounter = 3;
      publicCallRequests[2].callContext.sideEffectCounter = 4;

      const tx = mockTxWithPartialState({
        numberOfNonRevertiblePublicCallRequests: 2,
        numberOfRevertiblePublicCallRequests: 1,
        publicCallRequests,
      });

      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;
      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[0],
          contractStorageUpdateRequests: [
            new ContractStorageUpdateRequest(contractSlotA, fr(0x101), 11, baseContractAddress),
          ],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x102), 12, baseContractAddress),
                new ContractStorageUpdateRequest(contractSlotB, fr(0x151), 13, baseContractAddress),
              ],
            }).build(),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              revertReason: new SimulationError('Simulation Failed', []),
            }).build(),
          ],
        }).build(),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[2],
        }).build(),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotC, fr(0x201), 14, baseContractAddress),
              ],
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
      const publicCallRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      publicCallRequests[0].callContext.sideEffectCounter = 2;
      publicCallRequests[1].callContext.sideEffectCounter = 3;
      publicCallRequests[2].callContext.sideEffectCounter = 4;

      const tx = mockTxWithPartialState({
        numberOfNonRevertiblePublicCallRequests: 2,
        numberOfRevertiblePublicCallRequests: 1,
        publicCallRequests,
      });

      const teardownGas = tx.data.constants.txContext.gasSettings.getTeardownLimits();
      const teardownResultSettings = { startGasLeft: teardownGas, endGasLeft: teardownGas };

      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;
      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[0],
          contractStorageUpdateRequests: [
            new ContractStorageUpdateRequest(contractSlotA, fr(0x101), 11, baseContractAddress),
          ],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x102), 12, baseContractAddress),
                new ContractStorageUpdateRequest(contractSlotB, fr(0x151), 13, baseContractAddress),
              ],
            }).build(),
          ],
        }).build(),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[2],
        }).build(),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              revertReason: new SimulationError('Simulation Failed', []),
            }).build(teardownResultSettings),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotC, fr(0x201), 14, baseContractAddress),
              ],
            }).build(teardownResultSettings),
          ],
        }).build(teardownResultSettings),
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
      const publicCallRequests: PublicCallRequest[] = [
        baseContractAddressSeed,
        baseContractAddressSeed,
        baseContractAddressSeed,
      ].map(makePublicCallRequest);
      publicCallRequests[0].callContext.sideEffectCounter = 2;
      publicCallRequests[1].callContext.sideEffectCounter = 3;
      publicCallRequests[2].callContext.sideEffectCounter = 4;

      const tx = mockTxWithPartialState({
        numberOfNonRevertiblePublicCallRequests: 2,
        numberOfRevertiblePublicCallRequests: 1,
        publicCallRequests,
      });

      const gasLimits = Gas.from({ l1Gas: 1e9, l2Gas: 1e9, daGas: 1e9 });
      const teardownGas = Gas.from({ l1Gas: 1e7, l2Gas: 1e7, daGas: 1e7 });
      tx.data.constants.txContext.gasSettings = GasSettings.from({
        gasLimits: gasLimits,
        teardownGasLimits: teardownGas,
        inclusionFee: new Fr(1e4),
        maxFeesPerGas: { feePerDaGas: new Fr(10), feePerL1Gas: new Fr(10), feePerL2Gas: new Fr(10) },
      });

      // Private kernel tail to public pushes teardown gas allocation into revertible gas used
      tx.data.forPublic!.end.gasUsed = teardownGas;
      tx.data.forPublic!.endNonRevertibleData.gasUsed = Gas.empty();

      const contractSlotA = fr(0x100);
      const contractSlotB = fr(0x150);
      const contractSlotC = fr(0x200);

      let simulatorCallCount = 0;

      const initialGas = gasLimits.sub(teardownGas);
      const afterSetupGas = initialGas.sub(Gas.from({ l2Gas: 1e6 }));
      const afterAppGas = afterSetupGas.sub(Gas.from({ l2Gas: 2e6, daGas: 2e6 }));
      const afterTeardownGas = teardownGas.sub(Gas.from({ l2Gas: 3e6, daGas: 3e6 }));

      // Total gas used is the sum of teardown gas allocation plus all expenditures along the way,
      // without including the gas used in the teardown phase (since that's consumed entirely up front).
      const expectedTotalGasUsed = { l2Gas: 1e7 + 1e6 + 2e6, daGas: 1e7 + 2e6, l1Gas: 1e7 };

      // Inclusion fee plus block gas fees times total gas used
      const expectedTxFee = 1e4 + (1e7 + 1e6 + 2e6) * 1 + (1e7 + 2e6) * 1 + 1e7 * 1;
      const transactionFee = new Fr(expectedTxFee);

      const simulatorResults: PublicExecutionResult[] = [
        // Setup
        PublicExecutionResultBuilder.fromPublicCallRequest({ request: publicCallRequests[0] }).build({
          startGasLeft: initialGas,
          endGasLeft: afterSetupGas,
        }),

        // App Logic
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[2],
          contractStorageUpdateRequests: [
            new ContractStorageUpdateRequest(contractSlotA, fr(0x101), 14, baseContractAddress),
            new ContractStorageUpdateRequest(contractSlotB, fr(0x151), 15, baseContractAddress),
          ],
        }).build({
          startGasLeft: afterSetupGas,
          endGasLeft: afterAppGas,
        }),

        // Teardown
        PublicExecutionResultBuilder.fromPublicCallRequest({
          request: publicCallRequests[1],
          nestedExecutions: [
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x101), 11, baseContractAddress),
                new ContractStorageUpdateRequest(contractSlotC, fr(0x201), 12, baseContractAddress),
              ],
            }).build({ startGasLeft: teardownGas, endGasLeft: teardownGas, transactionFee }),
            PublicExecutionResultBuilder.fromFunctionCall({
              from: publicCallRequests[1].contractAddress,
              tx: makeFunctionCall(baseContractAddress, makeSelector(5)),
              contractStorageUpdateRequests: [
                new ContractStorageUpdateRequest(contractSlotA, fr(0x102), 13, baseContractAddress),
              ],
            }).build({ startGasLeft: teardownGas, endGasLeft: teardownGas, transactionFee }),
          ],
        }).build({
          startGasLeft: teardownGas,
          endGasLeft: afterTeardownGas,
          transactionFee,
        }),
      ];

      publicExecutor.simulate.mockImplementation(execution => {
        if (simulatorCallCount < simulatorResults.length) {
          const result = simulatorResults[simulatorCallCount++];
          return Promise.resolve(result);
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

      const expectedSimulateCall = (availableGas: Partial<FieldsOf<Gas>>, txFee: number) => [
        expect.anything(), // PublicExecution
        expect.anything(), // GlobalVariables
        Gas.from(availableGas),
        expect.anything(), // TxContext
        new Fr(txFee),
        expect.anything(), // SideEffectCounter
      ];

      expect(publicExecutor.simulate).toHaveBeenCalledTimes(3);
      expect(publicExecutor.simulate).toHaveBeenNthCalledWith(1, ...expectedSimulateCall(initialGas, 0));
      expect(publicExecutor.simulate).toHaveBeenNthCalledWith(2, ...expectedSimulateCall(afterSetupGas, 0));
      expect(publicExecutor.simulate).toHaveBeenNthCalledWith(3, ...expectedSimulateCall(teardownGas, expectedTxFee));

      expect(publicWorldStateDB.checkpoint).toHaveBeenCalledTimes(3);
      expect(publicWorldStateDB.rollbackToCheckpoint).toHaveBeenCalledTimes(0);
      expect(publicWorldStateDB.commit).toHaveBeenCalledTimes(1);
      expect(publicWorldStateDB.rollbackToCommit).toHaveBeenCalledTimes(0);

      expect(processed[0].data.end.gasUsed).toEqual(Gas.from(expectedTotalGasUsed));

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
