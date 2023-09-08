import {
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CircuitsWasm,
  Fr,
  GlobalVariables,
  KernelCircuitPublicInputs,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  Proof,
  PublicDataUpdateRequest,
  RootRollupPublicInputs,
  makeTuple,
  range,
} from '@aztec/circuits.js';
import { computeBlockHashWithGlobals, computeContractLeaf } from '@aztec/circuits.js/abis';
import {
  fr,
  makeBaseOrMergeRollupPublicInputs,
  makeNewContractData,
  makePrivateKernelPublicInputsFinal,
  makeProof,
  makePublicCallRequest,
  makeRootRollupPublicInputs,
} from '@aztec/circuits.js/factories';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { to2Fields } from '@aztec/foundation/serialize';
import {
  ContractData,
  ExtendedContractData,
  L2Block,
  L2BlockL2Logs,
  MerkleTreeId,
  PublicDataWrite,
  Tx,
  TxL2Logs,
  makeEmptyLogs,
  mockTx,
} from '@aztec/types';
import { MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { MockProxy, mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import flatMap from 'lodash.flatmap';
import times from 'lodash.times';
import { type MemDown, default as memdown } from 'memdown';

import { VerificationKeys, getVerificationKeys } from '../mocks/verification_keys.js';
import { EmptyRollupProver } from '../prover/empty.js';
import { RollupProver } from '../prover/index.js';
import {
  ProcessedTx,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricTreeRoots,
  makeProcessedTx,
} from '../sequencer/processed_tx.js';
import { getHistoricBlockData } from '../sequencer/utils.js';
import { RollupSimulator } from '../simulator/index.js';
import { WasmRollupCircuitSimulator } from '../simulator/rollup.js';
import { SoloBlockBuilder } from './solo_block_builder.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('sequencer/solo_block_builder', () => {
  let builder: SoloBlockBuilder;
  let builderDb: MerkleTreeOperations;
  let expectsDb: MerkleTreeOperations;
  let vks: VerificationKeys;

  let simulator: MockProxy<RollupSimulator>;
  let prover: MockProxy<RollupProver>;

  let blockNumber: number;
  let baseRollupOutputLeft: BaseOrMergeRollupPublicInputs;
  let baseRollupOutputRight: BaseOrMergeRollupPublicInputs;
  let rootRollupOutput: RootRollupPublicInputs;
  let mockL1ToL2Messages: Fr[];

  let wasm: CircuitsWasm;

  let globalVariables: GlobalVariables;

  const emptyProof = new Proof(Buffer.alloc(32, 0));

  const chainId = Fr.ZERO;
  const version = Fr.ZERO;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  beforeEach(async () => {
    blockNumber = 3;
    globalVariables = new GlobalVariables(chainId, version, new Fr(blockNumber), Fr.ZERO);

    builderDb = await MerkleTrees.new(levelup(createMemDown())).then(t => t.asLatest());
    expectsDb = await MerkleTrees.new(levelup(createMemDown())).then(t => t.asLatest());
    vks = getVerificationKeys();
    simulator = mock<RollupSimulator>();
    prover = mock<RollupProver>();
    builder = new SoloBlockBuilder(builderDb, vks, simulator, prover);

    // Create mock l1 to L2 messages
    mockL1ToL2Messages = new Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n));

    // Create mock outputs for simulator
    baseRollupOutputLeft = makeBaseOrMergeRollupPublicInputs(0, globalVariables);
    baseRollupOutputRight = makeBaseOrMergeRollupPublicInputs(0, globalVariables);
    rootRollupOutput = makeRootRollupPublicInputs(0, globalVariables);

    // Set up mocks
    prover.getBaseRollupProof.mockResolvedValue(emptyProof);
    prover.getRootRollupProof.mockResolvedValue(emptyProof);
    simulator.baseRollupCircuit
      .mockResolvedValueOnce(baseRollupOutputLeft)
      .mockResolvedValueOnce(baseRollupOutputRight);
    simulator.rootRollupCircuit.mockResolvedValue(rootRollupOutput);
  }, 20_000);

  const makeEmptyProcessedTx = async () => {
    const historicTreeRoots = await getHistoricBlockData(builderDb);
    return makeEmptyProcessedTxFromHistoricTreeRoots(historicTreeRoots, chainId, version);
  };

  // Updates the expectedDb trees based on the new commitments, contracts, and nullifiers from these txs
  const updateExpectedTreesFromTxs = async (txs: ProcessedTx[]) => {
    const newContracts = flatMap(txs, tx => tx.data.end.newContracts.map(n => computeContractLeaf(wasm, n)));
    for (const [tree, leaves] of [
      [MerkleTreeId.PRIVATE_DATA_TREE, flatMap(txs, tx => tx.data.end.newCommitments.map(l => l.toBuffer()))],
      [MerkleTreeId.CONTRACT_TREE, newContracts.map(x => x.toBuffer())],
    ] as const) {
      await expectsDb.appendLeaves(tree, leaves);
    }
    await expectsDb.batchInsert(
      MerkleTreeId.NULLIFIER_TREE,
      flatMap(txs, tx => tx.data.end.newNullifiers.map(x => x.toBuffer())),
      BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
    );
    for (const write of txs.flatMap(tx => tx.data.end.publicDataUpdateRequests)) {
      await expectsDb.updateLeaf(MerkleTreeId.PUBLIC_DATA_TREE, write.newValue.toBuffer(), write.leafIndex.value);
    }
  };

  const updateL1ToL2MessagesTree = async (l1ToL2Messages: Fr[]) => {
    const asBuffer = l1ToL2Messages.map(m => m.toBuffer());
    await expectsDb.appendLeaves(MerkleTreeId.L1_TO_L2_MESSAGES_TREE, asBuffer);
  };

  const updateHistoricBlocksTree = async () => {
    const blockHash = computeBlockHashWithGlobals(
      wasm,
      globalVariables,
      rootRollupOutput.endPrivateDataTreeSnapshot.root,
      rootRollupOutput.endNullifierTreeSnapshot.root,
      rootRollupOutput.endContractTreeSnapshot.root,
      rootRollupOutput.endL1ToL2MessageTreeSnapshot.root,
      rootRollupOutput.endPublicDataTreeRoot,
    );
    await expectsDb.appendLeaves(MerkleTreeId.BLOCKS_TREE, [blockHash.toBuffer()]);
  };

  const getTreeSnapshot = async (tree: MerkleTreeId) => {
    const treeInfo = await expectsDb.getTreeInfo(tree);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  };

  const buildMockSimulatorInputs = async () => {
    const kernelOutput = makePrivateKernelPublicInputsFinal();
    kernelOutput.constants.blockData = await getHistoricBlockData(expectsDb);

    const tx = await makeProcessedTx(
      new Tx(
        kernelOutput,
        emptyProof,
        makeEmptyLogs(),
        makeEmptyLogs(),
        times(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makePublicCallRequest),
        [ExtendedContractData.random()],
      ),
    );

    const txsLeft = [tx, await makeEmptyProcessedTx()];
    const txsRight = [await makeEmptyProcessedTx(), await makeEmptyProcessedTx()];

    // Calculate what would be the tree roots after the txs from the first base rollup land and update mock circuit output
    await updateExpectedTreesFromTxs(txsLeft);
    baseRollupOutputLeft.endContractTreeSnapshot = await getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    baseRollupOutputLeft.endNullifierTreeSnapshot = await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    baseRollupOutputLeft.endPrivateDataTreeSnapshot = await getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    baseRollupOutputLeft.endPublicDataTreeRoot = (await getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE)).root;

    // Same for the two txs on the right
    await updateExpectedTreesFromTxs(txsRight);
    baseRollupOutputRight.endContractTreeSnapshot = await getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    baseRollupOutputRight.endNullifierTreeSnapshot = await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    baseRollupOutputRight.endPrivateDataTreeSnapshot = await getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    baseRollupOutputRight.endPublicDataTreeRoot = (await getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE)).root;

    // Update l1 to l2 data tree
    // And update the root trees now to create proper output to the root rollup circuit
    await updateL1ToL2MessagesTree(mockL1ToL2Messages);
    rootRollupOutput.endContractTreeSnapshot = await getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    rootRollupOutput.endNullifierTreeSnapshot = await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    rootRollupOutput.endPrivateDataTreeSnapshot = await getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    rootRollupOutput.endPublicDataTreeRoot = (await getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE)).root;

    rootRollupOutput.endL1ToL2MessageTreeSnapshot = await getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGES_TREE);

    // Calculate block hash
    rootRollupOutput.globalVariables = globalVariables;
    await updateHistoricBlocksTree();
    rootRollupOutput.endHistoricBlocksTreeSnapshot = await getTreeSnapshot(MerkleTreeId.BLOCKS_TREE);

    const txs = [...txsLeft, ...txsRight];

    const newNullifiers = flatMap(txs, tx => tx.data.end.newNullifiers);
    const newCommitments = flatMap(txs, tx => tx.data.end.newCommitments);
    const newContracts = flatMap(txs, tx => tx.data.end.newContracts).map(cd => computeContractLeaf(wasm, cd));
    const newContractData = flatMap(txs, tx => tx.data.end.newContracts).map(
      n => new ContractData(n.contractAddress, n.portalContractAddress),
    );
    const newPublicDataWrites = flatMap(txs, tx =>
      tx.data.end.publicDataUpdateRequests.map(t => new PublicDataWrite(t.leafIndex, t.newValue)),
    );
    const newL2ToL1Msgs = flatMap(txs, tx => tx.data.end.newL2ToL1Msgs);
    const newEncryptedLogs = new L2BlockL2Logs(txs.map(tx => tx.encryptedLogs || new TxL2Logs([])));
    const newUnencryptedLogs = new L2BlockL2Logs(txs.map(tx => tx.unencryptedLogs || new TxL2Logs([])));

    const l2Block = L2Block.fromFields({
      number: blockNumber,
      globalVariables,
      startPrivateDataTreeSnapshot: rootRollupOutput.startPrivateDataTreeSnapshot,
      endPrivateDataTreeSnapshot: rootRollupOutput.endPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot: rootRollupOutput.startNullifierTreeSnapshot,
      endNullifierTreeSnapshot: rootRollupOutput.endNullifierTreeSnapshot,
      startContractTreeSnapshot: rootRollupOutput.startContractTreeSnapshot,
      endContractTreeSnapshot: rootRollupOutput.endContractTreeSnapshot,
      startPublicDataTreeRoot: rootRollupOutput.startPublicDataTreeRoot,
      endPublicDataTreeRoot: rootRollupOutput.endPublicDataTreeRoot,
      startL1ToL2MessageTreeSnapshot: rootRollupOutput.startL1ToL2MessageTreeSnapshot,
      endL1ToL2MessageTreeSnapshot: rootRollupOutput.endL1ToL2MessageTreeSnapshot,
      startHistoricBlocksTreeSnapshot: rootRollupOutput.startHistoricBlocksTreeSnapshot,
      endHistoricBlocksTreeSnapshot: rootRollupOutput.endHistoricBlocksTreeSnapshot,
      newCommitments,
      newNullifiers,
      newContracts,
      newContractData,
      newPublicDataWrites,
      newL1ToL2Messages: mockL1ToL2Messages,
      newL2ToL1Msgs,
      newEncryptedLogs,
      newUnencryptedLogs,
    });

    const callDataHash = l2Block.getCalldataHash();
    const high = Fr.fromBuffer(callDataHash.slice(0, 16));
    const low = Fr.fromBuffer(callDataHash.slice(16, 32));

    rootRollupOutput.calldataHash = [high, low];

    return txs;
  };

  describe('mock simulator', () => {
    beforeEach(() => {
      // Create instance to test
      builder = new SoloBlockBuilder(builderDb, vks, simulator, prover);
    });

    it('builds an L2 block using mock simulator', async () => {
      // Assemble a fake transaction
      const txs = await buildMockSimulatorInputs();

      // Actually build a block!
      const [l2Block, proof] = await builder.buildL2Block(globalVariables, txs, mockL1ToL2Messages);

      expect(l2Block.number).toEqual(blockNumber);
      expect(proof).toEqual(emptyProof);
    }, 20000);

    it('rejects if too many l1 to l2 messages are provided', async () => {
      // Assemble a fake transaction
      const txs = await buildMockSimulatorInputs();
      const l1ToL2Messages = new Array(100).fill(new Fr(0n));
      await expect(builder.buildL2Block(globalVariables, txs, l1ToL2Messages)).rejects.toThrow();
    });
  });

  describe('circuits simulator', () => {
    beforeEach(async () => {
      const simulator = await WasmRollupCircuitSimulator.new();
      const prover = new EmptyRollupProver();
      builder = new SoloBlockBuilder(builderDb, vks, simulator, prover);
    });

    const makeContractDeployProcessedTx = async (seed = 0x1) => {
      const tx = await makeEmptyProcessedTx();
      tx.data.end.newContracts = [makeNewContractData(seed + 0x1000)];
      return tx;
    };

    const makeBloatedProcessedTx = async (seed = 0x1) => {
      const tx = mockTx(seed);
      const kernelOutput = KernelCircuitPublicInputs.empty();
      kernelOutput.constants.blockData = await getHistoricBlockData(builderDb);
      kernelOutput.end.publicDataUpdateRequests = makeTuple(
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        i => new PublicDataUpdateRequest(fr(i), fr(0), fr(i + 10)),
        seed + 0x500,
      );

      const processedTx = await makeProcessedTx(tx, kernelOutput, makeProof());

      processedTx.data.end.newCommitments = makeTuple(MAX_NEW_COMMITMENTS_PER_TX, fr, seed + 0x100);
      processedTx.data.end.newNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, fr, seed + 0x200);
      processedTx.data.end.newNullifiers[tx.data.end.newNullifiers.length - 1] = Fr.ZERO;
      processedTx.data.end.newL2ToL1Msgs = makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x300);
      processedTx.data.end.newContracts = [makeNewContractData(seed + 0x1000)];
      processedTx.data.end.encryptedLogsHash = to2Fields(L2Block.computeKernelLogsHash(processedTx.encryptedLogs));
      processedTx.data.end.unencryptedLogsHash = to2Fields(L2Block.computeKernelLogsHash(processedTx.unencryptedLogs));

      return processedTx;
    };

    it.each([
      [0, 4],
      [1, 4],
      [4, 4],
      [0, 16],
      [16, 16],
    ] as const)(
      'builds an L2 block with %i contract deploy txs and %i txs total',
      async (deployCount: number, totalCount: number) => {
        const contractTreeBefore = await builderDb.getTreeInfo(MerkleTreeId.CONTRACT_TREE);

        const txs = [
          ...(await Promise.all(times(deployCount, makeContractDeployProcessedTx))),
          ...(await Promise.all(times(totalCount - deployCount, makeEmptyProcessedTx))),
        ];

        const [l2Block] = await builder.buildL2Block(globalVariables, txs, mockL1ToL2Messages);
        expect(l2Block.number).toEqual(blockNumber);

        await updateExpectedTreesFromTxs(txs);
        const contractTreeAfter = await builderDb.getTreeInfo(MerkleTreeId.CONTRACT_TREE);

        if (deployCount > 0) {
          expect(contractTreeAfter.root).not.toEqual(contractTreeBefore.root);
        }

        const expectedContractTreeAfter = await expectsDb.getTreeInfo(MerkleTreeId.CONTRACT_TREE).then(t => t.root);
        expect(contractTreeAfter.root).toEqual(expectedContractTreeAfter);
        expect(contractTreeAfter.size).toEqual(BigInt(totalCount));
      },
      30000,
    );

    it('builds an empty L2 block', async () => {
      const txs = await Promise.all([
        makeEmptyProcessedTx(),
        makeEmptyProcessedTx(),
        makeEmptyProcessedTx(),
        makeEmptyProcessedTx(),
      ]);

      const [l2Block] = await builder.buildL2Block(globalVariables, txs, mockL1ToL2Messages);
      expect(l2Block.number).toEqual(blockNumber);
    }, 10_000);

    it('builds a mixed L2 block', async () => {
      // Ensure that each transaction has unique (non-intersecting nullifier values)
      const txs = await Promise.all([
        makeBloatedProcessedTx(1 * MAX_NEW_NULLIFIERS_PER_TX),
        makeBloatedProcessedTx(2 * MAX_NEW_NULLIFIERS_PER_TX),
        makeBloatedProcessedTx(3 * MAX_NEW_NULLIFIERS_PER_TX),
        makeBloatedProcessedTx(4 * MAX_NEW_NULLIFIERS_PER_TX),
      ]);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const [l2Block] = await builder.buildL2Block(globalVariables, txs, l1ToL2Messages);
      expect(l2Block.number).toEqual(blockNumber);
    }, 20_000);

    // This test specifically tests nullifier values which previously caused e2e_private_token test to fail
    it('e2e_private_token edge case regression test on nullifier values', async () => {
      const simulator = await WasmRollupCircuitSimulator.new();
      const prover = new EmptyRollupProver();
      builder = new SoloBlockBuilder(builderDb, vks, simulator, prover);
      // update the starting tree
      const updateVals = Array(4 * MAX_NEW_NULLIFIERS_PER_TX).fill(0n);
      updateVals[0] = 19777494491628650244807463906174285795660759352776418619064841306523677458742n;
      updateVals[1] = 10246291467305176436335175657884940686778521321101740385288169037814567547848n;

      // new added values
      const tx = await makeEmptyProcessedTx();
      tx.data.end.newNullifiers[0] = new Fr(
        10336601644835972678500657502133589897705389664587188571002640950065546264856n,
      );
      tx.data.end.newNullifiers[1] = new Fr(
        17490072961923661940560522096125238013953043065748521735636170028491723851741n,
      );

      const txs = [tx, await makeEmptyProcessedTx(), await makeEmptyProcessedTx(), await makeEmptyProcessedTx()];

      // Must be built after the txs are created
      await builderDb.batchInsert(
        MerkleTreeId.NULLIFIER_TREE,
        updateVals.map(v => toBufferBE(v, 32)),
        BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
      );

      const [l2Block] = await builder.buildL2Block(globalVariables, txs, mockL1ToL2Messages);

      expect(l2Block.number).toEqual(blockNumber);
    }, 10000);
  });

  // describe("Input guard tests", () => {
  // })
});
