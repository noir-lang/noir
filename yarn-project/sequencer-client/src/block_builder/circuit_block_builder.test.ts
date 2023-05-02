import {
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  CircuitsWasm,
  Fr,
  KernelCircuitPublicInputs,
  PublicDataRead,
  PublicDataTransition,
  RootRollupPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import {
  fr,
  makeBaseRollupPublicInputs,
  makeKernelPublicInputs,
  makeNewContractData,
  makeProof,
  makeRootRollupPublicInputs,
} from '@aztec/circuits.js/factories';
import { toBufferBE } from '@aztec/foundation';
import { PublicDataWrite, Tx } from '@aztec/types';
import { MerkleTreeId, MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';
import { MockProxy, mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import flatMap from 'lodash.flatmap';
import times from 'lodash.times';
import { default as memdown, type MemDown } from 'memdown';
import { makeEmptyUnverifiedData, makePublicTx } from '../mocks/tx.js';
import { VerificationKeys, getVerificationKeys } from '../mocks/verification_keys.js';
import { EmptyRollupProver } from '../prover/empty.js';
import { RollupProver } from '../prover/index.js';
import {
  ProcessedTx,
  makeEmptyProcessedTx as makeEmptyProcessedTxFromHistoricTreeRoots,
  makeProcessedTx,
} from '../sequencer/processed_tx.js';
import { getCombinedHistoricTreeRoots } from '../sequencer/utils.js';
import { RollupSimulator } from '../simulator/index.js';
import { WasmRollupCircuitSimulator } from '../simulator/rollup.js';
import { CircuitBlockBuilder } from './circuit_block_builder.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('sequencer/circuit_block_builder', () => {
  let builder: CircuitBlockBuilder;
  let builderDb: MerkleTreeOperations;
  let expectsDb: MerkleTreeOperations;
  let vks: VerificationKeys;

  let simulator: MockProxy<RollupSimulator>;
  let prover: MockProxy<RollupProver>;

  let blockNumber: number;
  let baseRollupOutputLeft: BaseOrMergeRollupPublicInputs;
  let baseRollupOutputRight: BaseOrMergeRollupPublicInputs;
  let rootRollupOutput: RootRollupPublicInputs;

  let wasm: CircuitsWasm;

  const emptyProof = new UInt8Vector(Buffer.alloc(32, 0));

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  beforeEach(async () => {
    blockNumber = 3;
    builderDb = await MerkleTrees.new(levelup(createMemDown())).then(t => t.asLatest());
    expectsDb = await MerkleTrees.new(levelup(createMemDown())).then(t => t.asLatest());
    vks = getVerificationKeys();
    simulator = mock<RollupSimulator>();
    prover = mock<RollupProver>();
    builder = new CircuitBlockBuilder(builderDb, vks, simulator, prover);

    // Create mock outputs for simualator
    baseRollupOutputLeft = makeBaseRollupPublicInputs();
    baseRollupOutputRight = makeBaseRollupPublicInputs();
    rootRollupOutput = makeRootRollupPublicInputs();

    // Set up mocks
    prover.getBaseRollupProof.mockResolvedValue(emptyProof);
    prover.getRootRollupProof.mockResolvedValue(emptyProof);
    simulator.baseRollupCircuit
      .mockResolvedValueOnce(baseRollupOutputLeft)
      .mockResolvedValueOnce(baseRollupOutputRight);
    simulator.rootRollupCircuit.mockResolvedValue(rootRollupOutput);
  }, 20_000);

  const makeEmptyProcessedTx = async () => {
    const historicTreeRoots = await getCombinedHistoricTreeRoots(builderDb);
    return makeEmptyProcessedTxFromHistoricTreeRoots(historicTreeRoots);
  };

  // Updates the expectedDb trees based on the new commitments, contracts, and nullifiers from these txs
  const updateExpectedTreesFromTxs = async (txs: ProcessedTx[]) => {
    const newContracts = flatMap(txs, tx => tx.data.end.newContracts.map(n => computeContractLeaf(wasm, n)));
    for (const [tree, leaves] of [
      [MerkleTreeId.PRIVATE_DATA_TREE, flatMap(txs, tx => tx.data.end.newCommitments.map(l => l.toBuffer()))],
      [MerkleTreeId.CONTRACT_TREE, newContracts.map(x => x.toBuffer())],
      [MerkleTreeId.NULLIFIER_TREE, flatMap(txs, tx => tx.data.end.newNullifiers.map(l => l.toBuffer()))],
    ] as const) {
      await expectsDb.appendLeaves(tree, leaves);
    }
    for (const write of txs.flatMap(tx => tx.data.end.stateTransitions)) {
      await expectsDb.updateLeaf(MerkleTreeId.PUBLIC_DATA_TREE, write.newValue.toBuffer(), write.leafIndex.value);
    }
  };

  const getTreeSnapshot = async (tree: MerkleTreeId) => {
    const treeInfo = await expectsDb.getTreeInfo(tree);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  };

  describe('mock simulator', () => {
    it('builds an L2 block using mock simulator', async () => {
      // Create instance to test
      builder = new CircuitBlockBuilder(builderDb, vks, simulator, prover);

      // Assemble a fake transaction
      const kernelOutput = makeKernelPublicInputs();
      kernelOutput.constants.historicTreeRoots = await getCombinedHistoricTreeRoots(expectsDb);
      const tx = await makeProcessedTx(Tx.createPrivate(kernelOutput, emptyProof, makeEmptyUnverifiedData()));

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

      // And update the root trees now to create proper output to the root rollup circuit
      await expectsDb.updateRootsTrees();
      rootRollupOutput.endContractTreeSnapshot = await getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
      rootRollupOutput.endNullifierTreeSnapshot = await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
      rootRollupOutput.endPrivateDataTreeSnapshot = await getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
      rootRollupOutput.endPublicDataTreeRoot = (await getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE)).root;
      rootRollupOutput.endTreeOfHistoricContractTreeRootsSnapshot = await getTreeSnapshot(
        MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
      );
      rootRollupOutput.endTreeOfHistoricPrivateDataTreeRootsSnapshot = await getTreeSnapshot(
        MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
      );

      // Actually build a block!
      const txs = [tx, await makeEmptyProcessedTx(), await makeEmptyProcessedTx(), await makeEmptyProcessedTx()];
      const [l2Block, proof] = await builder.buildL2Block(blockNumber, txs);

      expect(l2Block.number).toEqual(blockNumber);
      expect(proof).toEqual(emptyProof);
    }, 20000);

    // For varying orders of insertions assert the local batch insertion generator creates the correct proofs
    it.each([
      [[16, 15, 14, 13, 0, 0, 0, 0]],
      [[13, 14, 15, 16, 0, 0, 0, 0]],
      [[1234, 98, 0, 0, 99999, 88, 54, 0]],
      [[97, 98, 10, 0, 99999, 88, 100001, 9000000]],
    ] as const)('performs nullifier tree batch insertion correctly', async nullifiers => {
      const leaves = nullifiers.map(i => toBufferBE(BigInt(i), 32));
      await expectsDb.appendLeaves(MerkleTreeId.NULLIFIER_TREE, leaves);

      builder = new CircuitBlockBuilder(builderDb, vks, simulator, prover);

      await builder.performBaseRollupBatchInsertionProofs(leaves);

      const expected = await expectsDb.getTreeInfo(MerkleTreeId.NULLIFIER_TREE);
      const actual = await builderDb.getTreeInfo(MerkleTreeId.NULLIFIER_TREE);
      expect(actual).toEqual(expected);
    });
  });

  describe('circuits simulator', () => {
    beforeEach(async () => {
      const simulator = await WasmRollupCircuitSimulator.new();
      const prover = new EmptyRollupProver();
      builder = new CircuitBlockBuilder(builderDb, vks, simulator, prover);
    });

    const makeContractDeployProcessedTx = async (seed = 0x1) => {
      const tx = await makeEmptyProcessedTx();
      tx.data.end.newContracts = [makeNewContractData(seed + 0x1000)];
      return tx;
    };

    const makePublicCallProcessedTx = async (seed = 0x1) => {
      const publicTx = makePublicTx(seed);
      const kernelOutput = KernelCircuitPublicInputs.empty();
      kernelOutput.end.stateReads[0] = new PublicDataRead(fr(1), fr(0));
      kernelOutput.end.stateTransitions[0] = new PublicDataTransition(fr(2), fr(0), fr(12));
      kernelOutput.constants.historicTreeRoots = await getCombinedHistoricTreeRoots(builderDb);
      return await makeProcessedTx(publicTx, kernelOutput, makeProof());
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

        const [l2Block] = await builder.buildL2Block(blockNumber, txs);
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
      10000,
    );

    it('builds an L2 block with private and public txs', async () => {
      const txs = await Promise.all([
        makePublicCallProcessedTx(),
        makeContractDeployProcessedTx(),
        makeEmptyProcessedTx(),
        makeEmptyProcessedTx(),
      ]);

      const [l2Block] = await builder.buildL2Block(blockNumber, txs);
      expect(l2Block.number).toEqual(blockNumber);
      expect(l2Block.newPublicDataWrites[0]).toEqual(new PublicDataWrite(fr(2), fr(12)));
      await updateExpectedTreesFromTxs(txs);
    });

    // This test specifically tests nullifier values which previously caused e2e_zk_token test to fail
    it('e2e_zk_token edge case regression test on nullifier values', async () => {
      const simulator = await WasmRollupCircuitSimulator.new();
      const prover = new EmptyRollupProver();
      builder = new CircuitBlockBuilder(builderDb, vks, simulator, prover);
      // update the starting tree
      const updateVals = Array(16).fill(0n);
      updateVals[0] = 19777494491628650244807463906174285795660759352776418619064841306523677458742n;
      updateVals[1] = 10246291467305176436335175657884940686778521321101740385288169037814567547848n;

      await builderDb.appendLeaves(
        MerkleTreeId.NULLIFIER_TREE,
        updateVals.map(v => toBufferBE(v, 32)),
      );

      // new added values
      const tx = await makeEmptyProcessedTx();
      tx.data.end.newNullifiers[0] = new Fr(
        10336601644835972678500657502133589897705389664587188571002640950065546264856n,
      );
      tx.data.end.newNullifiers[1] = new Fr(
        17490072961923661940560522096125238013953043065748521735636170028491723851741n,
      );
      const txs = [tx, await makeEmptyProcessedTx(), await makeEmptyProcessedTx(), await makeEmptyProcessedTx()];

      const [l2Block] = await builder.buildL2Block(blockNumber, txs);
      expect(l2Block.number).toEqual(blockNumber);
    }, 10000);
  });
});
