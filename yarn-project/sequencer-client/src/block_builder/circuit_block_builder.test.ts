import {
  AppendOnlyTreeSnapshot,
  BaseRollupInputs,
  BaseOrMergeRollupPublicInputs,
  CircuitsWasm,
  Fr,
  RootRollupPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import {
  makeBaseRollupPublicInputs,
  makeNewContractData,
  makePrivateKernelPublicInputs,
  makeRootRollupPublicInputs,
} from '@aztec/circuits.js/factories';
import { Tx } from '@aztec/types';
import { MerkleTreeId, MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';
import { MockProxy, mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import flatMap from 'lodash.flatmap';
import { default as memdown } from 'memdown';
import { makeEmptyTx, makeEmptyUnverifiedData } from '../mocks/tx.js';
import { VerificationKeys, getVerificationKeys } from '../mocks/verification_keys.js';
import { EmptyProver } from '../prover/empty.js';
import { Prover } from '../prover/index.js';
import { Simulator } from '../simulator/index.js';
import { WasmCircuitSimulator } from '../simulator/wasm.js';
import { CircuitBlockBuilder } from './circuit_block_builder.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import times from 'lodash.times';

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore
export const createMemDown = () => memdown();

describe('sequencer/circuit_block_builder', () => {
  let builder: TestSubject;
  let builderDb: MerkleTreeOperations;
  let expectsDb: MerkleTreeOperations;
  let vks: VerificationKeys;

  let simulator: MockProxy<Simulator>;
  let prover: MockProxy<Prover>;

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
    simulator = mock<Simulator>();
    prover = mock<Prover>();
    builder = new TestSubject(builderDb, vks, simulator, prover);

    // Populate root trees with first roots from the empty trees
    // TODO: Should this be responsibility of the MerkleTreeDb init?
    await updateRootTrees();

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

  const updateRootTrees = async () => {
    for (const [newTree, rootTree] of [
      [MerkleTreeId.DATA_TREE, MerkleTreeId.DATA_TREE_ROOTS_TREE],
      [MerkleTreeId.CONTRACT_TREE, MerkleTreeId.CONTRACT_TREE_ROOTS_TREE],
    ] as const) {
      const newTreeInfo = await expectsDb.getTreeInfo(newTree);
      await expectsDb.appendLeaves(rootTree, [newTreeInfo.root]);
    }
  };

  // Updates the expectedDb trees based on the new commitments, contracts, and nullifiers from these txs
  const updateExpectedTreesFromTxs = async (txs: Tx[]) => {
    const newContracts = flatMap(txs, tx => tx.data.end.newContracts.map(n => computeContractLeaf(wasm, n)));
    for (const [tree, leaves] of [
      [MerkleTreeId.DATA_TREE, flatMap(txs, tx => tx.data.end.newCommitments.map(l => l.toBuffer()))],
      [MerkleTreeId.CONTRACT_TREE, newContracts.map(x => x.toBuffer())],
      [MerkleTreeId.NULLIFIER_TREE, flatMap(txs, tx => tx.data.end.newNullifiers.map(l => l.toBuffer()))],
    ] as const) {
      await expectsDb.appendLeaves(tree, leaves);
    }
  };

  const getTreeSnapshot = async (tree: MerkleTreeId) => {
    const treeInfo = await expectsDb.getTreeInfo(tree);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  };

  const setTxOldTreeRoots = async (tx: Tx) => {
    for (const [name, id] of [
      ['privateDataTreeRoot', MerkleTreeId.DATA_TREE],
      ['contractTreeRoot', MerkleTreeId.CONTRACT_TREE],
      ['nullifierTreeRoot', MerkleTreeId.NULLIFIER_TREE],
    ] as const) {
      tx.data.constants.oldTreeRoots[name] = Fr.fromBuffer((await builderDb.getTreeInfo(id)).root);
    }
  };

  describe('mock simulator', () => {
    it('builds an L2 block using mock simulator', async () => {
      // Create instance to test
      builder = new TestSubject(builderDb, vks, simulator, prover);
      await builder.updateRootTrees();

      // Assemble a fake transaction, we'll tweak some fields below
      const tx = new Tx(makePrivateKernelPublicInputs(), emptyProof, makeEmptyUnverifiedData());
      const txsLeft = [tx, makeEmptyTx()];
      const txsRight = [makeEmptyTx(), makeEmptyTx()];

      // Set tree roots to proper values in the tx
      await setTxOldTreeRoots(tx);

      // Calculate what would be the tree roots after the txs from the first base rollup land and update mock circuit output
      await updateExpectedTreesFromTxs(txsLeft);
      baseRollupOutputLeft.endContractTreeSnapshot = await getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
      baseRollupOutputLeft.endNullifierTreeSnapshot = await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
      baseRollupOutputLeft.endPrivateDataTreeSnapshot = await getTreeSnapshot(MerkleTreeId.DATA_TREE);

      // Same for the two txs on the right
      await updateExpectedTreesFromTxs(txsRight);
      baseRollupOutputRight.endContractTreeSnapshot = await getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
      baseRollupOutputRight.endNullifierTreeSnapshot = await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
      baseRollupOutputRight.endPrivateDataTreeSnapshot = await getTreeSnapshot(MerkleTreeId.DATA_TREE);

      // And update the root trees now to create proper output to the root rollup circuit
      await updateRootTrees();
      rootRollupOutput.endContractTreeSnapshot = await getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
      rootRollupOutput.endNullifierTreeSnapshot = await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
      rootRollupOutput.endPrivateDataTreeSnapshot = await getTreeSnapshot(MerkleTreeId.DATA_TREE);
      rootRollupOutput.endTreeOfHistoricContractTreeRootsSnapshot = await getTreeSnapshot(
        MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
      );
      rootRollupOutput.endTreeOfHistoricPrivateDataTreeRootsSnapshot = await getTreeSnapshot(
        MerkleTreeId.DATA_TREE_ROOTS_TREE,
      );

      // Actually build a block!
      const txs = [tx, makeEmptyTx(), makeEmptyTx(), makeEmptyTx()];
      const [l2Block, proof] = await builder.buildL2Block(blockNumber, txs);

      expect(l2Block.number).toEqual(blockNumber);
      expect(proof).toEqual(emptyProof);
    }, 20000);
  });

  describe('circuits simulator', () => {
    beforeEach(async () => {
      const simulator = await WasmCircuitSimulator.new();
      const prover = new EmptyProver();
      builder = new TestSubject(builderDb, vks, simulator, prover);
      await builder.updateRootTrees();
    });

    const makeContractDeployTx = async (seed = 0x1) => {
      const tx = makeEmptyTx();
      await setTxOldTreeRoots(tx);
      tx.data.end.newContracts = [makeNewContractData(seed + 0x1000)];
      return tx;
    };

    it.each([
      [0, 4],
      [1, 4],
      [4, 4],
    ] as const)(
      'builds an L2 block with %i contract deploy txs and %i txs total',
      async (deployCount: number, totalCount: number) => {
        const contractTreeBefore = await builderDb.getTreeInfo(MerkleTreeId.CONTRACT_TREE);

        const txs = [
          ...(await Promise.all(times(deployCount, makeContractDeployTx))),
          ...times(totalCount - deployCount, makeEmptyTx),
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
  });
});

// Test subject class that exposes internal functions for testing
class TestSubject extends CircuitBlockBuilder {
  public buildBaseRollupInput(tx1: Tx, tx2: Tx): Promise<BaseRollupInputs> {
    return super.buildBaseRollupInput(tx1, tx2);
  }

  public updateRootTrees(): Promise<void> {
    return super.updateRootTrees();
  }
}
