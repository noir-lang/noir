import {
  AppendOnlyTreeSnapshot,
  BaseRollupInputs,
  BaseOrMergeRollupPublicInputs,
  CircuitsWasm,
  ConstantBaseRollupData,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  MembershipWitness,
  NullifierLeafPreimage,
  NULLIFIER_TREE_HEIGHT,
  PreviousKernelData,
  PreviousRollupData,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
  ROLLUP_VK_TREE_HEIGHT,
  RootRollupInputs,
  RootRollupPublicInputs,
  UInt8Vector,
  VK_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { Fr, createDebugLogger, toBigIntBE } from '@aztec/foundation';
import { LeafData, SiblingPath } from '@aztec/merkle-tree';
import { Tx } from '@aztec/tx';
import { MerkleTreeId, MerkleTreeOperations } from '@aztec/world-state';
import flatMap from 'lodash.flatmap';
import times from 'lodash.times';
import { makeEmptyTx } from '../deps/tx.js';
import { VerificationKeys } from '../deps/verification_keys.js';
import { Proof, Prover } from '../prover/index.js';
import { Simulator } from '../simulator/index.js';
import { ContractData, L2Block } from '@aztec/l2-block';
import { computeContractLeaf } from '@aztec/circuits.js/abis';

const frToBigInt = (fr: Fr) => toBigIntBE(fr.toBuffer());
const bigintToFr = (num: bigint) => new Fr(num);
const bigintToNum = (num: bigint) => Number(num);

// Denotes fields that are not used now, but will be in the future
const FUTURE_FR = new Fr(0n);
const FUTURE_NUM = 0;

// Denotes fields that should be deleted
const DELETE_FR = new Fr(0n);
const DELETE_NUM = 0;

/**
 * All of the data required for the circuit compute and verify nullifiers
 */
export interface LowNullifierWitnessData {
  /**
   * Preimage of the low nullifier that proves non membership
   */
  preimage: NullifierLeafPreimage;
  /**
   * Sibling path to prove membership of low nullifier
   */
  siblingPath: SiblingPath;
  /**
   * The index of low nullifier
   */
  index: bigint;
}

export class CircuitPoweredBlockBuilder {
  constructor(
    protected db: MerkleTreeOperations,
    protected vks: VerificationKeys,
    protected simulator: Simulator,
    protected prover: Prover,
    protected wasm: CircuitsWasm,
    protected debug = createDebugLogger('aztec:sequencer'),
  ) {}

  public async buildL2Block(blockNumber: number, tx: Tx): Promise<[L2Block, UInt8Vector]> {
    const [
      startPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
    ] = await Promise.all(
      [
        MerkleTreeId.DATA_TREE,
        MerkleTreeId.NULLIFIER_TREE,
        MerkleTreeId.CONTRACT_TREE,
        MerkleTreeId.DATA_TREE_ROOTS_TREE,
        MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
      ].map(tree => this.getTreeSnapshot(tree)),
    );

    // We fill the tx batch with empty txs, we process only one tx at a time for now
    const txs = [tx, makeEmptyTx(), makeEmptyTx(), makeEmptyTx()];
    const [circuitsOutput, proof] = await this.runCircuits(txs);

    const {
      endPrivateDataTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
    } = circuitsOutput;

    // Collect all new nullifiers, commitments, and contracts from all txs in this block
    const newNullifiers = flatMap(txs, tx => tx.data.end.newNullifiers);
    const newCommitments = flatMap(txs, tx => tx.data.end.newCommitments);
    const newContracts = await Promise.all(
      flatMap(txs, tx => tx.data.end.newContracts).map(async cd => await computeContractLeaf(this.wasm, cd)),
    );
    const newContractData = flatMap(txs, tx => tx.data.end.newContracts).map(
      n => new ContractData(n.contractAddress, n.portalContractAddress),
    );

    const l2Block = L2Block.fromFields({
      number: blockNumber,
      startPrivateDataTreeSnapshot,
      endPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      endNullifierTreeSnapshot,
      startContractTreeSnapshot,
      endContractTreeSnapshot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
      newCommitments,
      newNullifiers,
      newContracts,
      newContractData,
    });

    return [l2Block, proof];
  }

  protected async getTreeSnapshot(id: MerkleTreeId): Promise<AppendOnlyTreeSnapshot> {
    const treeInfo = await this.db.getTreeInfo(id);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  }

  protected async runCircuits(txs: Tx[]): Promise<[RootRollupPublicInputs, Proof]> {
    const [tx1, tx2, tx3, tx4] = txs;

    // Simulate both base rollup circuits, updating the data, contract, and nullifier trees in the process
    this.debug(`Running left base rollup simulator`);
    const [baseRollupInputLeft, baseRollupOutputLeft] = await this.baseRollupCircuit(tx1, tx2);
    this.debug(`Running right base rollup simulator`);
    const [baseRollupInputRight, baseRollupOutputRight] = await this.baseRollupCircuit(tx3, tx4);

    // Get the proofs for them in parallel (faked for now)
    this.debug(`Running base rollup circuit provers`);
    const [baseRollupProofLeft, baseRollupProofRight] = await Promise.all([
      this.prover.getBaseRollupProof(baseRollupInputLeft, baseRollupOutputLeft),
      this.prover.getBaseRollupProof(baseRollupInputRight, baseRollupOutputRight),
    ]);

    // Get the input for the root rollup circuit based on the base rollup ones
    this.debug(`Producing root rollup inputs`);
    const rootInput = await this.getRootRollupInput(
      baseRollupOutputLeft,
      baseRollupProofLeft,
      baseRollupOutputRight,
      baseRollupProofRight,
    );

    // Simulate and get proof for the root circuit
    this.debug(`Running root rollup simulator`);
    const rootOutput = await this.simulator.rootRollupCircuit(rootInput);
    this.debug(`Running root rollup circuit prover`);
    const rootProof = await this.prover.getRootRollupProof(rootInput, rootOutput);

    // Update the root trees with the latest data and contract tree roots,
    // and validate them against the output of the root circuit simulation
    this.debug(`Updating and validating root trees`);
    await this.updateRootTrees();
    await this.validateRootOutput(rootOutput);

    return [rootOutput, rootProof];
  }

  protected async baseRollupCircuit(tx1: Tx, tx2: Tx) {
    const rollupInput = await this.buildBaseRollupInput(tx1, tx2);
    const rollupOutput = await this.simulator.baseRollupCircuit(rollupInput);
    await this.validateTrees(rollupOutput);
    return [rollupInput, rollupOutput] as const;
  }

  // Updates our roots trees with the new generated trees after the rollup updates
  protected async updateRootTrees() {
    for (const [newTree, rootTree] of [
      [MerkleTreeId.DATA_TREE, MerkleTreeId.DATA_TREE_ROOTS_TREE],
      [MerkleTreeId.CONTRACT_TREE, MerkleTreeId.CONTRACT_TREE_ROOTS_TREE],
    ] as const) {
      const newTreeInfo = await this.db.getTreeInfo(newTree);
      await this.db.appendLeaves(rootTree, [newTreeInfo.root]);
    }
  }

  // Validate that the new roots we calculated from manual insertions match the outputs of the simulation
  protected async validateTrees(rollupOutput: BaseOrMergeRollupPublicInputs | RootRollupPublicInputs) {
    await Promise.all([
      this.validateTree(rollupOutput, MerkleTreeId.CONTRACT_TREE, 'Contract'),
      this.validateTree(rollupOutput, MerkleTreeId.DATA_TREE, 'PrivateData'),
      this.validateTree(rollupOutput, MerkleTreeId.NULLIFIER_TREE, 'Nullifier'),
    ]);
  }

  // Validate that the roots of all local trees match the output of the root circuit simulation
  protected async validateRootOutput(rootOutput: RootRollupPublicInputs) {
    await Promise.all([
      this.validateTrees(rootOutput),
      this.validateRootTree(rootOutput, MerkleTreeId.CONTRACT_TREE_ROOTS_TREE, 'Contract'),
      this.validateRootTree(rootOutput, MerkleTreeId.DATA_TREE_ROOTS_TREE, 'PrivateData'),
    ]);
  }

  // Helper for validating a roots tree against a circuit simulation output
  protected async validateRootTree(
    rootOutput: RootRollupPublicInputs,
    treeId: MerkleTreeId,
    name: 'Contract' | 'PrivateData',
  ) {
    const localTree = await this.getTreeSnapshot(treeId);
    const simulatedTree = rootOutput[`endTreeOfHistoric${name}TreeRootsSnapshot`];
    this.validateSimulatedTree(localTree, simulatedTree, name, `Roots ${name}`);
  }

  // Helper for validating a non-roots tree against a circuit simulation output
  protected async validateTree(
    output: BaseOrMergeRollupPublicInputs | RootRollupPublicInputs,
    treeId: MerkleTreeId,
    name: 'PrivateData' | 'Contract' | 'Nullifier',
  ) {
    const localTree = await this.getTreeSnapshot(treeId);
    const simulatedTree = output[`end${name}TreeSnapshot`];
    this.validateSimulatedTree(localTree, simulatedTree, name);
  }

  // Helper for comparing two trees snapshots
  protected validateSimulatedTree(
    localTree: AppendOnlyTreeSnapshot,
    simulatedTree: AppendOnlyTreeSnapshot,
    name: string,
    label?: string,
  ) {
    if (!simulatedTree.root.toBuffer().equals(localTree.root.toBuffer())) {
      throw new Error(`${label ?? name} tree root mismatch (local ${localTree.root}, simulated ${simulatedTree.root})`);
    }
    if (simulatedTree.nextAvailableLeafIndex !== localTree.nextAvailableLeafIndex) {
      throw new Error(
        `${label ?? name} tree next available leaf index mismatch (local ${
          localTree.nextAvailableLeafIndex
        }, simulated ${simulatedTree.nextAvailableLeafIndex})`,
      );
    }
  }

  // Builds the inputs for the root rollup circuit, without making any changes to trees
  protected async getRootRollupInput(
    rollupOutputLeft: BaseOrMergeRollupPublicInputs,
    rollupProofLeft: Proof,
    rollupOutputRight: BaseOrMergeRollupPublicInputs,
    rollupProofRight: Proof,
  ) {
    const previousRollupData: RootRollupInputs['previousRollupData'] = [
      this.getPreviousRollupDataFromBaseRollup(rollupOutputLeft, rollupProofLeft),
      this.getPreviousRollupDataFromBaseRollup(rollupOutputRight, rollupProofRight),
    ];

    const getRootTreeSiblingPath = async (treeId: MerkleTreeId) => {
      // TODO: Synchronize these operations into the tree db to avoid race conditions
      const { size } = await this.db.getTreeInfo(treeId);
      // TODO: Check for off-by-one errors
      const path = await this.db.getSiblingPath(treeId, size);
      return path.data.map(b => Fr.fromBuffer(b));
    };

    const newHistoricContractDataTreeRootSiblingPath = await getRootTreeSiblingPath(
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
    );
    const newHistoricPrivateDataTreeRootSiblingPath = await getRootTreeSiblingPath(MerkleTreeId.DATA_TREE_ROOTS_TREE);

    return RootRollupInputs.from({
      previousRollupData,
      newHistoricContractDataTreeRootSiblingPath,
      newHistoricPrivateDataTreeRootSiblingPath,
    });
  }

  protected getPreviousRollupDataFromBaseRollup(rollupOutput: BaseOrMergeRollupPublicInputs, rollupProof: Proof) {
    return new PreviousRollupData(
      rollupOutput,
      rollupProof,
      this.vks.baseRollupCircuit,

      // MembershipWitness for a VK tree to be implemented in the future
      FUTURE_NUM,
      new MembershipWitness(ROLLUP_VK_TREE_HEIGHT, FUTURE_NUM, Array(ROLLUP_VK_TREE_HEIGHT).fill(FUTURE_FR)),
    );
  }

  protected getKernelDataFor(tx: Tx) {
    return new PreviousKernelData(
      tx.data,
      tx.proof,

      // VK for the kernel circuit
      this.vks.kernelCircuit,

      // MembershipWitness for a VK tree to be implemented in the future
      FUTURE_NUM,
      Array(VK_TREE_HEIGHT).fill(FUTURE_FR),
    );
  }

  // Scan a tree searching for a specific value and return a membership witness proof for it
  protected async getMembershipWitnessFor<N extends number>(
    value: Fr,
    treeId: MerkleTreeId,
    height: N,
  ): Promise<MembershipWitness<N>> {
    // If this is an empty tx, then just return zeroes
    if (value.value === 0n) return this.makeEmptyMembershipWitness(height);

    const index = await this.db.findLeafIndex(treeId, value.toBuffer());
    if (index === undefined) {
      throw new Error(`Leaf with value ${value} not found in tree ${treeId}`);
    }
    const path = await this.db.getSiblingPath(treeId, index);
    // TODO: Check conversion from bigint to number
    return new MembershipWitness(
      height,
      Number(index),
      path.data.map(b => Fr.fromBuffer(b)),
    );
  }

  protected getContractMembershipWitnessFor(tx: Tx) {
    return this.getMembershipWitnessFor(
      tx.data.constants.oldTreeRoots.contractTreeRoot,
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
      CONTRACT_TREE_ROOTS_TREE_HEIGHT,
    );
  }

  protected getDataMembershipWitnessFor(tx: Tx) {
    return this.getMembershipWitnessFor(
      tx.data.constants.oldTreeRoots.privateDataTreeRoot,
      MerkleTreeId.DATA_TREE_ROOTS_TREE,
      PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
    );
  }

  protected async getConstantBaseRollupData(): Promise<ConstantBaseRollupData> {
    return ConstantBaseRollupData.from({
      baseRollupVkHash: DELETE_FR,
      mergeRollupVkHash: DELETE_FR,
      privateKernelVkTreeRoot: FUTURE_FR,
      publicKernelVkTreeRoot: FUTURE_FR,
      startTreeOfHistoricContractTreeRootsSnapshot: await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE_ROOTS_TREE),
      startTreeOfHistoricPrivateDataTreeRootsSnapshot: await this.getTreeSnapshot(MerkleTreeId.DATA_TREE_ROOTS_TREE),
      treeOfHistoricL1ToL2MsgTreeRootsSnapshot: new AppendOnlyTreeSnapshot(DELETE_FR, DELETE_NUM),
    });
  }

  protected async getLowNullifierInfo(nullifier: Fr) {
    // Return empty nullifier info for an empty tx
    if (nullifier.value === 0n) {
      return {
        index: 0,
        leafPreimage: new NullifierLeafPreimage(new Fr(0n), new Fr(0n), 0),
        witness: this.makeEmptyMembershipWitness(NULLIFIER_TREE_HEIGHT),
      };
    }

    const tree = MerkleTreeId.NULLIFIER_TREE;
    const prevValueIndex = await this.db.getPreviousValueIndex(tree, frToBigInt(nullifier));
    const prevValueInfo = await this.db.getLeafData(tree, prevValueIndex.index);
    if (!prevValueInfo) throw new Error(`Nullifier tree should have one initial leaf`);
    const prevValueSiblingPath = await this.db.getSiblingPath(tree, BigInt(prevValueIndex.index));

    return {
      index: prevValueIndex,
      leafPreimage: new NullifierLeafPreimage(
        bigintToFr(prevValueInfo.value),
        bigintToFr(prevValueInfo.nextValue),
        bigintToNum(prevValueInfo.nextIndex),
      ),
      witness: new MembershipWitness(
        NULLIFIER_TREE_HEIGHT,
        prevValueIndex.index,
        prevValueSiblingPath.data.map(b => Fr.fromBuffer(b)),
      ),
    };
  }

  protected async getSubtreeSiblingPath(treeId: MerkleTreeId, subtreeHeight: number): Promise<Fr[]> {
    // Get sibling path to the last leaf we inserted
    const lastLeafIndex = (await this.db.getTreeInfo(treeId).then(t => t.size)) - 1n;
    const fullSiblingPath = await this.db.getSiblingPath(treeId, lastLeafIndex);

    // Drop the first subtreeHeight items since we only care about the path to the subtree root
    return fullSiblingPath.data.slice(subtreeHeight).map(b => Fr.fromBuffer(b));
  }

  /**
   * Each base rollup needs to provide non membership / inclusion proofs for each of the nullifier.
   * This method will return membership proofs and perform partial node updates that will
   * allow the circuit to incrementally update the tree and perform a batch insertion.
   *
   * This offers massive circuit performance savings over doing incremental insertions.
   *
   * A description of the algorithm can be found here: https://colab.research.google.com/drive/1A0gizduSi4FIiIJZ8OylwIpO9-OTqV-R
   *
   * WARNING: This function has side effects, it will insert values into the tree.
   *
   * Assumptions:
   * 1. There are 8 nullifiers provided and they are either unique or empty. (denoted as 0)
   * 2. If kc 0 has 1 nullifier, and kc 1 has 3 nullifiers the layout will assume to be the sparse
   *   nullifier layout: [kc0-0, 0, 0, 0, kc1-0, kc1-1, kc1-2, 0]
   *
   * TODO: this implementation will change once the zero value is changed from h(0,0,0). Changes incoming over the next sprint
   * @param leaves Values to insert into the tree
   * @returns
   */
  public async performBaseRollupBatchInsertionProofs(leaves: Buffer[]): Promise<LowNullifierWitnessData[] | undefined> {
    // Keep track of the touched during batch insertion
    const touchedNodes: Set<number> = new Set<number>();

    // Return data
    const lowNullifierWitnesses: LowNullifierWitnessData[] = [];
    const dbInfo = await this.db.getTreeInfo(MerkleTreeId.NULLIFIER_TREE);
    const startInsertionIndex: bigint = dbInfo.size;
    let currInsertionIndex: bigint = startInsertionIndex;

    // Leaf data of hte leaves to be inserted
    const insertionSubtree: NullifierLeafPreimage[] = [];

    // Low nullifier membership proof sibling paths
    for (const leaf of leaves) {
      const newValue = toBigIntBE(leaf);
      const indexOfPrevious = await this.db.getPreviousValueIndex(MerkleTreeId.NULLIFIER_TREE, newValue);

      // NOTE: null values for nullfier leaves are being changed to 0n current impl is a hack
      // Default value
      const nullifierLeaf: NullifierLeafPreimage = new NullifierLeafPreimage(new Fr(newValue), new Fr(0n), 0);
      if (touchedNodes.has(indexOfPrevious.index) || newValue === 0n) {
        // If the node has already been touched, then we return an empty leaf and sibling path
        const emptySP = new SiblingPath();
        emptySP.data = Array(dbInfo.depth).fill(
          Buffer.from('0000000000000000000000000000000000000000000000000000000000000000', 'hex'),
        );
        const witness: LowNullifierWitnessData = {
          preimage: NullifierLeafPreimage.empty(),
          index: 0n,
          siblingPath: emptySP,
        };
        lowNullifierWitnesses.push(witness);
      } else {
        // If the node has not been touched, we update its low nullifier pointer, but we do NOT insert it yet, inserting it now
        // will alter non membership paths of the not yet inserted members
        // Insertion is done at the end once updates have already occurred.
        touchedNodes.add(indexOfPrevious.index);

        const lowNullifier = await this.db.getLeafData(MerkleTreeId.NULLIFIER_TREE, indexOfPrevious.index);

        // If no low nullifier can be found, abort - this means the nullifier is invalid
        // in some way (it should not happen)
        if (lowNullifier === undefined) {
          return undefined;
        }

        const lowNullifierPreimage = new NullifierLeafPreimage(
          new Fr(lowNullifier.value),
          new Fr(lowNullifier.nextValue),
          Number(lowNullifier.nextIndex),
        );

        // Get sibling path for existence of the old leaf
        const siblingPath = await this.db.getSiblingPath(MerkleTreeId.NULLIFIER_TREE, BigInt(indexOfPrevious.index));

        // Update the running paths
        const witness = {
          preimage: lowNullifierPreimage,
          index: BigInt(indexOfPrevious.index),
          siblingPath: siblingPath,
        };
        lowNullifierWitnesses.push(witness);

        // Update subtree insertion leaf from null data
        nullifierLeaf.nextIndex = lowNullifierPreimage.nextIndex;
        nullifierLeaf.nextValue = lowNullifierPreimage.nextValue;

        // Update the current low nullifier
        lowNullifier.nextIndex = currInsertionIndex;
        lowNullifier.nextValue = BigInt(newValue);

        // Update the old leaf in the tree
        await this.db.updateLeaf(MerkleTreeId.NULLIFIER_TREE, lowNullifier, BigInt(indexOfPrevious.index));
      }

      // increment insertion index
      currInsertionIndex++;
      insertionSubtree.push(nullifierLeaf);
    }

    // Create insertion subtree and forcefully insert in series
    // Here we calculate the pointers for the inserted values, if they have not already been updated
    for (let i = 0; i < leaves.length; i++) {
      const newValue = new Fr(toBigIntBE(leaves[i]));

      if (newValue.isZero()) continue;

      // We have already fetched the new low nullifier for this leaf, so we can set its low nullifier
      const lowNullifier = lowNullifierWitnesses[i].preimage;
      // If the lowNullifier is 0, then we check the previous leaves for the low nullifier leaf
      if (lowNullifier.leafValue.isZero() && lowNullifier.nextValue.isZero() && lowNullifier.nextIndex === 0) {
        for (let j = 0; j < i; j++) {
          if (
            (insertionSubtree[j].nextValue > newValue && insertionSubtree[j].leafValue < newValue) ||
            (insertionSubtree[j].nextValue.isZero() && insertionSubtree[j].nextIndex === 0)
          ) {
            insertionSubtree[j].nextIndex = Number(startInsertionIndex) + i;
            insertionSubtree[j].nextValue = newValue;
          }
        }
      }
    }

    // For each calculated new leaf, we insert it into the tree at the next position
    for (let i = 0; i < insertionSubtree.length; i++) {
      const asLeafData: LeafData = {
        value: insertionSubtree[i].leafValue.value,
        nextValue: insertionSubtree[i].nextValue.value,
        nextIndex: BigInt(insertionSubtree[i].nextIndex),
      };

      await this.db.updateLeaf(MerkleTreeId.NULLIFIER_TREE, asLeafData, startInsertionIndex + BigInt(i));
    }

    return lowNullifierWitnesses;
  }

  // Builds the base rollup inputs, updating the contract, nullifier, and data trees in the process
  protected async buildBaseRollupInput(tx1: Tx, tx2: Tx) {
    // Get trees info before any changes hit
    const constants = await this.getConstantBaseRollupData();
    const startNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const startContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const startPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.DATA_TREE);

    // Update the contract and data trees with the new items being inserted to get the new roots
    // that will be used by the next iteration of the base rollup circuit, skipping the empty ones
    const newContracts = await Promise.all(
      flatMap([tx1, tx2], tx => tx.data.end.newContracts.map(async cd => await computeContractLeaf(this.wasm, cd))),
    );
    const newCommitments = flatMap([tx1, tx2], tx => tx.data.end.newCommitments.map(x => x.toBuffer()));
    await this.db.appendLeaves(
      MerkleTreeId.CONTRACT_TREE,
      newContracts.map(x => x.toBuffer()),
    );

    await this.db.appendLeaves(MerkleTreeId.DATA_TREE, newCommitments);

    // Update the nullifier tree, capturing the low nullifier info for each individual operation
    const newNullifiers = [...tx1.data.end.newNullifiers, ...tx2.data.end.newNullifiers];

    const nullifierWitnesses = await this.performBaseRollupBatchInsertionProofs(newNullifiers.map(fr => fr.toBuffer()));
    if (nullifierWitnesses === undefined) {
      throw new Error(`Could not craft nullifier batch insertion proofs`);
    }
    // Extract witness objects from returned data
    const lowNullifierMembershipWitnesses = nullifierWitnesses.map(w =>
      MembershipWitness.fromBufferArray(Number(w.index), w.siblingPath.data),
    );

    // Get the subtree sibling paths for the circuit
    const newCommitmentsSubtreeSiblingPath = await this.getSubtreeSiblingPath(
      MerkleTreeId.DATA_TREE,
      BaseRollupInputs.PRIVATE_DATA_SUBTREE_HEIGHT,
    );
    const newContractsSubtreeSiblingPath = await this.getSubtreeSiblingPath(
      MerkleTreeId.CONTRACT_TREE,
      BaseRollupInputs.CONTRACT_SUBTREE_HEIGHT,
    );
    const newNullifiersSubtreeSiblingPath = await this.getSubtreeSiblingPath(
      MerkleTreeId.NULLIFIER_TREE,
      BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
    );

    return BaseRollupInputs.from({
      constants,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startPrivateDataTreeSnapshot,
      newCommitmentsSubtreeSiblingPath,
      newContractsSubtreeSiblingPath,
      newNullifiersSubtreeSiblingPath,
      lowNullifierLeafPreimages: nullifierWitnesses.map((w: LowNullifierWitnessData) => w.preimage),
      lowNullifierMembershipWitness: lowNullifierMembershipWitnesses,
      kernelData: [this.getKernelDataFor(tx1), this.getKernelDataFor(tx2)],
      historicContractsTreeRootMembershipWitnesses: [
        await this.getContractMembershipWitnessFor(tx1),
        await this.getContractMembershipWitnessFor(tx2),
      ],
      historicPrivateDataTreeRootMembershipWitnesses: [
        await this.getDataMembershipWitnessFor(tx1),
        await this.getDataMembershipWitnessFor(tx2),
      ],
    } as BaseRollupInputs);
  }

  protected makeEmptyMembershipWitness<N extends number>(height: N) {
    return new MembershipWitness(
      height,
      0,
      times(height, () => new Fr(0n)),
    );
  }
}
