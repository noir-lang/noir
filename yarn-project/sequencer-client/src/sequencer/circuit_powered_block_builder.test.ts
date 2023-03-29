import { inspectTree, MerkleTreeDb, MerkleTreeId, MerkleTrees } from '@aztec/world-state';
import { mock, MockProxy } from 'jest-mock-extended';
import { Prover } from '../prover/index.js';
import { Simulator } from '../simulator/index.js';
import { CircuitPoweredBlockBuilder } from './circuit_powered_block_builder.js';
import { VerificationKeys, getVerificationKeys } from './vks.js';
import { default as memdown } from 'memdown';
import { default as levelup } from 'levelup';
import {
  AppendOnlyTreeSnapshot,
  BaseRollupInputs,
  BaseRollupPublicInputs,
  Fr,
  RootRollupPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import { Tx } from '@aztec/tx';
import {
  makeBaseRollupPublicInputs,
  makePrivateKernelPublicInputs,
  makeRootRollupPublicInputs,
} from '@aztec/circuits.js/factories';
import { hashNewContractData, makeEmptyTx } from '../deps/tx.js';
import flatMap from 'lodash.flatmap';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore
export const createMemDown = () => memdown();

describe('sequencer/circuit_block_builder', () => {
  let builder: TestSubject;
  let builderDb: MerkleTreeDb;
  let expectsDb: MerkleTreeDb;
  let vks: VerificationKeys;
  let simulator: MockProxy<Simulator>;
  let prover: MockProxy<Prover>;

  let blockNumber: number;
  let baseRollupOutputLeft: BaseRollupPublicInputs;
  let baseRollupOutputRight: BaseRollupPublicInputs;
  let rootRollupOutput: RootRollupPublicInputs;

  let wasm: BarretenbergWasm;

  const emptyProof = new UInt8Vector(Buffer.alloc(32, 0));
  const emptyUnverifiedData = Buffer.alloc(0);

  beforeAll(async () => {
    wasm = new BarretenbergWasm();
    await wasm.init();
  });

  beforeEach(async () => {
    blockNumber = 3;
    builderDb = await MerkleTrees.new(levelup(createMemDown()));
    expectsDb = await MerkleTrees.new(levelup(createMemDown()));
    vks = getVerificationKeys();
    simulator = mock<Simulator>();
    prover = mock<Prover>();
    builder = new TestSubject(builderDb, blockNumber, vks, simulator, prover, wasm);

    // Populate root trees with first roots from the empty trees
    // TODO: Should this be responsibility of the MerkleTreeDb init?
    await updateRootTrees();
    await builder.updateRootTrees();

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
  });

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
    for (const [tree, leaves] of [
      [MerkleTreeId.DATA_TREE, flatMap(txs, tx => tx.data.end.newCommitments.map(l => l.toBuffer()))],
      [MerkleTreeId.CONTRACT_TREE, flatMap(txs, tx => tx.data.end.newContracts.map(n => hashNewContractData(wasm, n)))],
      [MerkleTreeId.NULLIFIER_TREE, flatMap(txs, tx => tx.data.end.newNullifiers.map(l => l.toBuffer()))],
    ] as const) {
      await expectsDb.appendLeaves(tree, leaves);
    }
  };

  const getTreeSnapshot = async (tree: MerkleTreeId) => {
    const treeInfo = await expectsDb.getTreeInfo(tree);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  };

  it('builds an L2 block', async () => {
    // Assemble a fake transaction, we'll tweak some fields below
    const tx = new Tx(makePrivateKernelPublicInputs(), emptyProof, emptyUnverifiedData);
    const txsLeft = [tx, makeEmptyTx()];
    const txsRight = [makeEmptyTx(), makeEmptyTx()];
    const txs = [...txsLeft, ...txsRight];

    // Set tree roots to proper values in the tx
    for (const [name, id] of [
      ['privateDataTreeRoot', MerkleTreeId.DATA_TREE],
      ['contractTreeRoot', MerkleTreeId.CONTRACT_TREE],
      ['nullifierTreeRoot', MerkleTreeId.NULLIFIER_TREE],
    ] as const) {
      tx.data.constants.oldTreeRoots[name] = Fr.fromBuffer((await builderDb.getTreeInfo(id)).root);
    }

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
    const [l2block, proof] = await builder.buildL2Block(tx);

    expect(l2block.number).toEqual(blockNumber);
    expect(proof).toEqual(emptyProof);
  });
});

// Test subject class that exposes internal functions for testing
class TestSubject extends CircuitPoweredBlockBuilder {
  public buildBaseRollupInput(tx1: Tx, tx2: Tx): Promise<BaseRollupInputs> {
    return super.buildBaseRollupInput(tx1, tx2);
  }

  public updateRootTrees(): Promise<void> {
    return super.updateRootTrees();
  }
}
