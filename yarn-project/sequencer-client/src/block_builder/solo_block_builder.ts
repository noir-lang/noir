import {
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  CircuitsWasm,
  ConstantBaseRollupData,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT,
  L1_TO_L2_MESSAGES_SUBTREE_HEIGHT,
  MembershipWitness,
  MergeRollupInputs,
  NULLIFIER_TREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NullifierLeafPreimage,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
  PreviousKernelData,
  PreviousRollupData,
  ROLLUP_VK_TREE_HEIGHT,
  RollupTypes,
  RootRollupInputs,
  RootRollupPublicInputs,
  VK_TREE_HEIGHT,
  VerificationKey,
} from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import { LeafData, SiblingPath } from '@aztec/merkle-tree';
import { MerkleTreeId, ContractData, L2Block, PublicDataWrite } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';
import chunk from 'lodash.chunk';
import flatMap from 'lodash.flatmap';
import times from 'lodash.times';
import { VerificationKeys } from '../mocks/verification_keys.js';
import { Proof, RollupProver } from '../prover/index.js';
import { RollupSimulator } from '../simulator/index.js';

import { ProcessedTx } from '../sequencer/processed_tx.js';
import { BlockBuilder } from './index.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { toFriendlyJSON } from '@aztec/circuits.js/utils';
import { AllowedTreeNames, OutputWithTreeSnapshot } from './types.js';

const frToBigInt = (fr: Fr) => toBigIntBE(fr.toBuffer());
const bigintToFr = (num: bigint) => new Fr(num);
const bigintToNum = (num: bigint) => Number(num);

// Denotes fields that are not used now, but will be in the future
const FUTURE_FR = new Fr(0n);
const FUTURE_NUM = 0;

// Denotes fields that should be deleted
const DELETE_FR = new Fr(0n);

/**
 * All of the data required for the circuit compute and verify nullifiers.
 */
export interface LowNullifierWitnessData {
  /**
   * Preimage of the low nullifier that proves non membership.
   */
  preimage: NullifierLeafPreimage;
  /**
   * Sibling path to prove membership of low nullifier.
   */
  siblingPath: SiblingPath;
  /**
   * The index of low nullifier.
   */
  index: bigint;
}

// Pre-compute empty nullifier witness
const EMPTY_LOW_NULLIFIER_WITNESS: LowNullifierWitnessData = {
  preimage: NullifierLeafPreimage.empty(),
  index: 0n,
  siblingPath: new SiblingPath(Array(NULLIFIER_TREE_HEIGHT).fill(toBufferBE(0n, 32))),
};

/**
 * Builds an L2 block out of a set of ProcessedTx's,
 * using the base, merge, and root rollup circuits.
 */
export class SoloBlockBuilder implements BlockBuilder {
  constructor(
    protected db: MerkleTreeOperations,
    protected vks: VerificationKeys,
    protected simulator: RollupSimulator,
    protected prover: RollupProver,
    protected debug = createDebugLogger('aztec:sequencer'),
  ) {}

  /**
   * Builds an L2 block with the given number containing the given txs, updating state trees.
   * @param blockNumber - Number of the block to create.
   * @param txs - Processed transactions to include in the block.
   * @param newL1ToL2Messages - L1 to L2 messages to be part of the block.
   * @returns The new L2 block and a correctness proof as returned by the root rollup circuit.
   */
  public async buildL2Block(
    blockNumber: number,
    txs: ProcessedTx[],
    newL1ToL2Messages: Fr[],
  ): Promise<[L2Block, Proof]> {
    const [
      startPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startPublicDataTreeSnapshot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
      startL1ToL2MessageTreeSnapshot,
      startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
    ] = await Promise.all(
      [
        MerkleTreeId.PRIVATE_DATA_TREE,
        MerkleTreeId.NULLIFIER_TREE,
        MerkleTreeId.CONTRACT_TREE,
        MerkleTreeId.PUBLIC_DATA_TREE,
        MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
        MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
        MerkleTreeId.L1_TO_L2_MESSAGES_TREE,
        MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE,
      ].map(tree => this.getTreeSnapshot(tree)),
    );

    // Check txs are good for processing
    this.validateTxs(txs);

    for (const tx of txs) {
      if (tx.isEmpty) {
        continue;
      }
      if (!tx.data.end.newNullifiers[tx.data.end.newNullifiers.length - 1].isZero()) {
        throw new Error(`Unable to insert tx hash as first nullifier`);
      }
      tx.data.end.newNullifiers = [
        Fr.fromBuffer(tx.hash.buffer),
        ...tx.data.end.newNullifiers.slice(0, KERNEL_NEW_NULLIFIERS_LENGTH - 1),
      ];
    }

    // We fill the tx batch with empty txs, we process only one tx at a time for now
    const [circuitsOutput, proof] = await this.runCircuits(txs, newL1ToL2Messages);

    const {
      endPrivateDataTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endPublicDataTreeRoot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
      endL1ToL2MessageTreeSnapshot,
      endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
    } = circuitsOutput;

    // Collect all new nullifiers, commitments, and contracts from all txs in this block
    const wasm = await CircuitsWasm.get();
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

    const l2Block = L2Block.fromFields({
      number: blockNumber,
      startPrivateDataTreeSnapshot,
      endPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      endNullifierTreeSnapshot,
      startContractTreeSnapshot,
      endContractTreeSnapshot,
      startPublicDataTreeRoot: startPublicDataTreeSnapshot.root,
      endPublicDataTreeRoot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
      startL1ToL2MessageTreeSnapshot,
      endL1ToL2MessageTreeSnapshot,
      startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      newCommitments,
      newNullifiers,
      newL2ToL1Msgs,
      newContracts,
      newContractData,
      newPublicDataWrites,
      newL1ToL2Messages,
    });

    if (!l2Block.getCalldataHash().equals(circuitsOutput.sha256CalldataHash())) {
      throw new Error(
        `Calldata hash mismatch, ${l2Block.getCalldataHash().toString('hex')} == ${circuitsOutput
          .sha256CalldataHash()
          .toString('hex')} `,
      );
    }

    return [l2Block, proof];
  }

  protected validateTxs(txs: ProcessedTx[]) {
    for (const tx of txs) {
      for (const historicTreeRoot of [
        'privateDataTreeRoot',
        'contractTreeRoot',
        'nullifierTreeRoot',
        'l1ToL2MessagesTreeRoot',
      ] as const) {
        if (tx.data.constants.historicTreeRoots.privateHistoricTreeRoots[historicTreeRoot].isZero()) {
          throw new Error(`Empty ${historicTreeRoot} for tx: ${toFriendlyJSON(tx)}`);
        }
      }
    }
  }

  protected async getTreeSnapshot(id: MerkleTreeId): Promise<AppendOnlyTreeSnapshot> {
    const treeInfo = await this.db.getTreeInfo(id);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  }

  protected async runCircuits(txs: ProcessedTx[], newL1ToL2Messages: Fr[]): Promise<[RootRollupPublicInputs, Proof]> {
    // Check that the length of the array of txs is a power of two
    // See https://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
    if (txs.length < 4 || (txs.length & (txs.length - 1)) !== 0) {
      throw new Error(`Length of txs for the block should be a power of two and at least four (got ${txs.length})`);
    }

    // Check that the number of new L1 to L2 messages is the same as the max
    if (newL1ToL2Messages.length !== NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP) {
      throw new Error(
        `Length of the l1 to l2 messages per block should be a constant ${NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP} (got ${newL1ToL2Messages.length})`,
      );
    }

    // Run the base rollup circuits for the txs
    const baseRollupOutputs: [BaseOrMergeRollupPublicInputs, Proof][] = [];
    for (const pair of chunk(txs, 2)) {
      const [tx1, tx2] = pair;
      baseRollupOutputs.push(await this.baseRollupCircuit(tx1, tx2));
    }

    // Run merge rollups in layers until we have only two outputs
    let mergeRollupInputs: [BaseOrMergeRollupPublicInputs, Proof][] = baseRollupOutputs;
    let mergeRollupOutputs: [BaseOrMergeRollupPublicInputs, Proof][] = [];
    while (mergeRollupInputs.length > 2) {
      for (const pair of chunk(mergeRollupInputs, 2)) {
        const [r1, r2] = pair;
        mergeRollupOutputs.push(await this.mergeRollupCircuit(r1, r2));
      }
      mergeRollupInputs = mergeRollupOutputs;
      mergeRollupOutputs = [];
    }

    // Run the root rollup with the last two merge rollups (or base, if no merge layers)
    const [mergeOutputLeft, mergeOutputRight] = mergeRollupInputs;
    return this.rootRollupCircuit(mergeOutputLeft, mergeOutputRight, newL1ToL2Messages);
  }

  protected async baseRollupCircuit(
    tx1: ProcessedTx,
    tx2: ProcessedTx,
  ): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    this.debug(`Running base rollup for ${tx1.hash} ${tx2.hash}`);
    const rollupInput = await this.buildBaseRollupInput(tx1, tx2);
    const rollupOutput = await this.simulator.baseRollupCircuit(rollupInput);
    await this.validateTrees(rollupOutput);
    const proof = await this.prover.getBaseRollupProof(rollupInput, rollupOutput);
    return [rollupOutput, proof];
  }

  protected async mergeRollupCircuit(
    left: [BaseOrMergeRollupPublicInputs, Proof],
    right: [BaseOrMergeRollupPublicInputs, Proof],
  ): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    const vk = this.getVerificationKey(left[0].rollupType);
    const mergeInputs = new MergeRollupInputs([
      this.getPreviousRollupDataFromPublicInputs(left[0], left[1], vk),
      this.getPreviousRollupDataFromPublicInputs(right[0], right[1], vk),
    ]);

    this.debug(`Running merge rollup circuit`);
    const output = await this.simulator.mergeRollupCircuit(mergeInputs);
    const proof = await this.prover.getMergeRollupProof(mergeInputs, output);
    return [output, proof];
  }

  protected getVerificationKey(type: RollupTypes) {
    switch (type) {
      case RollupTypes.Base:
        return this.vks.baseRollupCircuit;
      case RollupTypes.Merge:
        return this.vks.mergeRollupCircuit;
      default:
        throw new Error(`No verification key available for ${type}`);
    }
  }

  protected async rootRollupCircuit(
    left: [BaseOrMergeRollupPublicInputs, Proof],
    right: [BaseOrMergeRollupPublicInputs, Proof],
    newL1ToL2Messages: Fr[],
  ): Promise<[RootRollupPublicInputs, Proof]> {
    this.debug(`Running root rollup circuit`);
    const rootInput = await this.getRootRollupInput(...left, ...right, newL1ToL2Messages);

    // Update the local trees to include the new l1 to l2 messages
    await this.db.appendLeaves(
      MerkleTreeId.L1_TO_L2_MESSAGES_TREE,
      newL1ToL2Messages.map(m => m.toBuffer()),
    );

    // Simulate and get proof for the root circuit
    const rootOutput = await this.simulator.rootRollupCircuit(rootInput);

    const rootProof = await this.prover.getRootRollupProof(rootInput, rootOutput);

    // Update the root trees with the latest data and contract tree roots,
    // and validate them against the output of the root circuit simulation
    this.debug(`Updating and validating root trees`);
    await this.db.updateHistoricRootsTrees();
    await this.validateRootOutput(rootOutput);

    return [rootOutput, rootProof];
  }

  // Validate that the new roots we calculated from manual insertions match the outputs of the simulation
  protected async validateTrees(rollupOutput: BaseOrMergeRollupPublicInputs | RootRollupPublicInputs) {
    await Promise.all([
      this.validateTree(rollupOutput, MerkleTreeId.CONTRACT_TREE, 'Contract'),
      this.validateTree(rollupOutput, MerkleTreeId.PRIVATE_DATA_TREE, 'PrivateData'),
      this.validateTree(rollupOutput, MerkleTreeId.NULLIFIER_TREE, 'Nullifier'),
      this.validatePublicDataTreeRoot(rollupOutput),
    ]);
  }

  // Validate that the roots of all local trees match the output of the root circuit simulation
  protected async validateRootOutput(rootOutput: RootRollupPublicInputs) {
    await Promise.all([
      this.validateTrees(rootOutput),
      this.validateRootTree(rootOutput, MerkleTreeId.CONTRACT_TREE_ROOTS_TREE, 'Contract'),
      this.validateRootTree(rootOutput, MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE, 'PrivateData'),
      this.validateRootTree(rootOutput, MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE, 'L1ToL2Message'),
      this.validateTree(rootOutput, MerkleTreeId.L1_TO_L2_MESSAGES_TREE, 'L1ToL2Message'),
    ]);
  }

  // Helper for validating a roots tree against a circuit simulation output
  protected async validateRootTree(
    rootOutput: RootRollupPublicInputs,
    treeId: MerkleTreeId,
    name: 'Contract' | 'PrivateData' | 'L1ToL2Message',
  ) {
    const localTree = await this.getTreeSnapshot(treeId);
    const simulatedTree = rootOutput[`endTreeOfHistoric${name}TreeRootsSnapshot`];
    this.validateSimulatedTree(localTree, simulatedTree, name, `Roots ${name}`);
  }

  /**
   * Validates that the root of the public data tree matches the output of the circuit simulation.
   * @param output - The output of the circuit simulation.
   * Note: Public data tree is sparse, so the "next available leaf index" doesn't make sense there.
   *       For this reason we only validate root.
   */
  protected async validatePublicDataTreeRoot(output: BaseOrMergeRollupPublicInputs | RootRollupPublicInputs) {
    const localTree = await this.getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE);
    const simulatedTreeRoot = output[`endPublicDataTreeRoot`];

    if (!simulatedTreeRoot.toBuffer().equals(localTree.root.toBuffer())) {
      throw new Error(`PublicData tree root mismatch (local ${localTree.root}, simulated ${simulatedTreeRoot})`);
    }
  }

  // Helper for validating a non-roots tree against a circuit simulation output
  protected async validateTree<T extends BaseOrMergeRollupPublicInputs | RootRollupPublicInputs>(
    output: T,
    treeId: MerkleTreeId,
    name: AllowedTreeNames<T>,
  ) {
    if ('endL1ToL2MessageTreeSnapshot' in output && !(output instanceof RootRollupPublicInputs)) {
      throw new Error(`The name 'L1ToL2Message' can only be used when output is of type RootRollupPublicInputs`);
    }

    const localTree = await this.getTreeSnapshot(treeId);
    const simulatedTree = (output as OutputWithTreeSnapshot<T>)[`end${name}TreeSnapshot`];
    this.validateSimulatedTree(localTree, simulatedTree, name);
  }

  // Helper for comparing two trees snapshots
  protected validateSimulatedTree(
    localTree: AppendOnlyTreeSnapshot,
    simulatedTree: AppendOnlyTreeSnapshot,
    name: 'PrivateData' | 'Contract' | 'Nullifier' | 'L1ToL2Message',
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
    newL1ToL2Messages: Fr[],
  ) {
    const vk = this.getVerificationKey(rollupOutputLeft.rollupType);
    const previousRollupData: RootRollupInputs['previousRollupData'] = [
      this.getPreviousRollupDataFromPublicInputs(rollupOutputLeft, rollupProofLeft, vk),
      this.getPreviousRollupDataFromPublicInputs(rollupOutputRight, rollupProofRight, vk),
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
    const newHistoricPrivateDataTreeRootSiblingPath = await getRootTreeSiblingPath(
      MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
    );
    const newHistoricL1ToL2MessageTreeRootSiblingPath = await getRootTreeSiblingPath(
      MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE,
    );
    const newL1ToL2MessageTreeRootSiblingPath = await this.getSubtreeSiblingPath(
      MerkleTreeId.L1_TO_L2_MESSAGES_TREE,
      L1_TO_L2_MESSAGES_SUBTREE_HEIGHT,
    );

    // Get tree snapshots
    const startL1ToL2MessageTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGES_TREE);
    const startHistoricTreeL1ToL2MessageTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE,
    );

    return RootRollupInputs.from({
      previousRollupData,
      newHistoricContractDataTreeRootSiblingPath,
      newHistoricPrivateDataTreeRootSiblingPath,
      newL1ToL2Messages,
      newHistoricL1ToL2MessageTreeRootSiblingPath,
      newL1ToL2MessageTreeRootSiblingPath,
      startL1ToL2MessageTreeSnapshot,
      startHistoricTreeL1ToL2MessageTreeRootsSnapshot,
    });
  }

  protected getPreviousRollupDataFromPublicInputs(
    rollupOutput: BaseOrMergeRollupPublicInputs,
    rollupProof: Proof,
    vk: VerificationKey,
  ) {
    return new PreviousRollupData(
      rollupOutput,
      rollupProof,
      vk,

      // MembershipWitness for a VK tree to be implemented in the future
      FUTURE_NUM,
      new MembershipWitness(ROLLUP_VK_TREE_HEIGHT, BigInt(FUTURE_NUM), Array(ROLLUP_VK_TREE_HEIGHT).fill(FUTURE_FR)),
    );
  }

  protected getKernelDataFor(tx: ProcessedTx) {
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
      index,
      path.data.map(b => Fr.fromBuffer(b)),
    );
  }

  protected getContractMembershipWitnessFor(tx: ProcessedTx) {
    return this.getMembershipWitnessFor(
      tx.data.constants.historicTreeRoots.privateHistoricTreeRoots.contractTreeRoot,
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
      CONTRACT_TREE_ROOTS_TREE_HEIGHT,
    );
  }

  protected getDataMembershipWitnessFor(tx: ProcessedTx) {
    return this.getMembershipWitnessFor(
      tx.data.constants.historicTreeRoots.privateHistoricTreeRoots.privateDataTreeRoot,
      MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
      PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
    );
  }

  protected getL1ToL2MessageMembershipWitnessFor(tx: ProcessedTx) {
    return this.getMembershipWitnessFor(
      tx.data.constants.historicTreeRoots.privateHistoricTreeRoots.l1ToL2MessagesTreeRoot,
      MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE,
      L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT,
    );
  }

  protected async getConstantBaseRollupData(): Promise<ConstantBaseRollupData> {
    return ConstantBaseRollupData.from({
      baseRollupVkHash: DELETE_FR,
      mergeRollupVkHash: DELETE_FR,
      privateKernelVkTreeRoot: FUTURE_FR,
      publicKernelVkTreeRoot: FUTURE_FR,
      startTreeOfHistoricContractTreeRootsSnapshot: await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE_ROOTS_TREE),
      startTreeOfHistoricPrivateDataTreeRootsSnapshot: await this.getTreeSnapshot(
        MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
      ),
      startTreeOfHistoricL1ToL2MsgTreeRootsSnapshot: await this.getTreeSnapshot(
        MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE,
      ),
    });
  }

  protected async getLowNullifierInfo(nullifier: Fr) {
    // Return empty nullifier info for an empty tx
    if (nullifier.value === 0n) {
      return {
        index: 0,
        leafPreimage: NullifierLeafPreimage.empty(),
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
        BigInt(prevValueIndex.index),
        prevValueSiblingPath.data.map(b => Fr.fromBuffer(b)),
      ),
    };
  }

  protected async getSubtreeSiblingPath(treeId: MerkleTreeId, subtreeHeight: number): Promise<Fr[]> {
    const nextAvailableLeafIndex = await this.db.getTreeInfo(treeId).then(t => t.size);
    const fullSiblingPath = await this.db.getSiblingPath(treeId, nextAvailableLeafIndex);

    // Drop the first subtreeHeight items since we only care about the path to the subtree root
    return fullSiblingPath.data.slice(subtreeHeight).map(b => Fr.fromBuffer(b));
  }

  /* eslint-disable jsdoc/require-description-complete-sentence */
  /* The following doc block messes up with complete-sentence, so we just disable it */

  /**
   *
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
   * Algorithm overview
   *
   * In general, if we want to batch insert items, we first to update their low nullifier to point to them,
   * then batch insert all of the values as at once in the final step.
   * To update a low nullifier, we provide an insertion proof that the low nullifier currently exists to the
   * circuit, then update the low nullifier.
   * Updating this low nullifier will in turn change the root of the tree. Therefore future low nullifier insertion proofs
   * must be given against this new root.
   * As a result, each low nullifier membership proof will be provided against an intermediate tree state, each with differing
   * roots.
   *
   * This become tricky when two items that are being batch inserted need to update the same low nullifier, or need to use
   * a value that is part of the same batch insertion as their low nullifier. In this case a zero low nullifier path is given
   * to the circuit, and it must determine from the set of batch inserted values if the insertion is valid.
   *
   * The following example will illustrate attempting to insert 2,3,20,19 into a tree already containing 0,5,10,15
   *
   * The example will explore two cases. In each case the values low nullifier will exist within the batch insertion,
   * One where the low nullifier comes before the item in the set (2,3), and one where it comes after (20,19).
   *
   * The original tree:                       Pending insertion subtree
   *
   *  index     0       2       3       4         -       -       -       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         -       -       -       -
   *  nextIdx   1       2       3       0         -       -       -       -
   *  nextVal   5      10      15       0         -       -       -       -
   *
   *
   * Inserting 2: (happy path)
   * 1. Find the low nullifier (0) - provide inclusion proof
   * 2. Update its pointers
   * 3. Insert 2 into the pending subtree
   *
   *  index     0       2       3       4         5       -       -       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       -       -       -
   *  nextIdx   5       2       3       0         2       -       -       -
   *  nextVal   2      10      15       0         5       -       -       -
   *
   * Inserting 3: The low nullifier exists within the insertion current subtree
   * 1. When looking for the low nullifier for 3, we will receive 0 again as we have not inserted 2 into the main tree
   *    This is problematic, as we cannot use either 0 or 2 as our inclusion proof.
   *    Why cant we?
   *      - Index 0 has a val 0 and nextVal of 2. This is NOT enough to prove non inclusion of 2.
   *      - Our existing tree is in a state where we cannot prove non inclusion of 3.
   *    We do not provide a non inclusion proof to out circuit, but prompt it to look within the insertion subtree.
   * 2. Update pending insertion subtree
   * 3. Insert 3 into pending subtree
   *
   * (no inclusion proof provided)
   *  index     0       2       3       4         5       6       -       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       3       -       -
   *  nextIdx   5       2       3       0         6       2       -       -
   *  nextVal   2      10      15       0         3       5       -       -
   *
   * Inserting 20: (happy path)
   * 1. Find the low nullifier (15) - provide inculsion proof
   * 2. Update its pointers
   * 3. Insert 20 into the pending subtree
   *
   *  index     0       2       3       4         5       6       7       -
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       3      20       -
   *  nextIdx   5       2       3       7         6       2       0       -
   *  nextVal   2      10      15      20         3       5       0       -
   *
   * Inserting 19:
   * 1. In this case we can find a low nullifier, but we are updating a low nullifier that has already been updated
   *    We can provide an inclusion proof of this intermediate tree state.
   * 2. Update its pointers
   * 3. Insert 19 into the pending subtree
   *
   *  index     0       2       3       4         5       6       7       8
   *  -------------------------------------      ----------------------------
   *  val       0       5      10      15         2       3      20       19
   *  nextIdx   5       2       3       8         6       2       0       7
   *  nextVal   2      10      15      19         3       5       0       20
   *
   * Perform subtree insertion
   *
   *  index     0       2       3       4       5       6       7       8
   *  ---------------------------------------------------------------------
   *  val       0       5      10      15       2       3      20       19
   *  nextIdx   5       2       3       8       6       2       0       7
   *  nextVal   2      10      15      19       3       5       0       20
   *
   * TODO: this implementation will change once the zero value is changed from h(0,0,0). Changes incoming over the next sprint
   * @param leaves - Values to insert into the tree
   * @returns The witness data for the leaves to be updated when inserting the new ones.
   */
  public async performBaseRollupBatchInsertionProofs(
    leaves: Buffer[],
  ): Promise<[LowNullifierWitnessData[], Fr[]] | [undefined, Fr[]]> {
    /* eslint-enable */

    // Keep track of touched low nullifiers
    const touched = new Map<number, bigint[]>();

    // Accumulators
    const lowNullifierWitnesses: LowNullifierWitnessData[] = [];
    const pendingInsertionSubtree: NullifierLeafPreimage[] = [];

    // Start info
    const dbInfo = await this.db.getTreeInfo(MerkleTreeId.NULLIFIER_TREE);
    const startInsertionIndex: bigint = dbInfo.size;

    // Get insertion path for each leaf
    for (let i = 0; i < leaves.length; i++) {
      const newValue = toBigIntBE(leaves[i]);

      // Keep space and just insert zero values
      if (newValue === 0n) {
        pendingInsertionSubtree.push(NullifierLeafPreimage.empty());
        lowNullifierWitnesses.push(EMPTY_LOW_NULLIFIER_WITNESS);
        continue;
      }

      const indexOfPrevious = await this.db.getPreviousValueIndex(MerkleTreeId.NULLIFIER_TREE, newValue);

      // If a touched node has a value that is less greater than the current value
      const prevNodes = touched.get(indexOfPrevious.index);
      if (prevNodes && prevNodes.some(v => v < newValue)) {
        // check the pending low nullifiers for a low nullifier that works
        // This is the case where the next value is less than the pending
        for (let j = 0; j < pendingInsertionSubtree.length; j++) {
          if (pendingInsertionSubtree[j].leafValue.isZero()) continue;

          if (
            pendingInsertionSubtree[j].leafValue.value < newValue &&
            (pendingInsertionSubtree[j].nextValue.value > newValue || pendingInsertionSubtree[j].nextValue.isZero())
          ) {
            // add the new value to the pending low nullifiers
            const currentLeafLowNullifier = new NullifierLeafPreimage(
              new Fr(newValue),
              pendingInsertionSubtree[j].nextValue,
              Number(pendingInsertionSubtree[j].nextIndex),
            );

            pendingInsertionSubtree.push(currentLeafLowNullifier);

            // Update the pending low nullifier to point at the new value
            pendingInsertionSubtree[j].nextValue = new Fr(newValue);
            pendingInsertionSubtree[j].nextIndex = Number(startInsertionIndex) + i;

            break;
          }
        }

        // Any node updated in this space will need to calculate its low nullifier from a previously inserted value
        lowNullifierWitnesses.push(EMPTY_LOW_NULLIFIER_WITNESS);
      } else {
        // Update the touched mapping
        if (prevNodes) {
          prevNodes.push(newValue);
          touched.set(indexOfPrevious.index, prevNodes);
        } else {
          touched.set(indexOfPrevious.index, [newValue]);
        }

        // get the low nullifier
        const lowNullifier = await this.db.getLeafData(MerkleTreeId.NULLIFIER_TREE, indexOfPrevious.index);
        if (lowNullifier === undefined) {
          return [
            undefined,
            await this.getSubtreeSiblingPath(MerkleTreeId.NULLIFIER_TREE, BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT),
          ];
        }

        const lowNullifierPreimage = new NullifierLeafPreimage(
          new Fr(lowNullifier.value),
          new Fr(lowNullifier.nextValue),
          Number(lowNullifier.nextIndex),
        );
        const siblingPath = await this.db.getSiblingPath(MerkleTreeId.NULLIFIER_TREE, BigInt(indexOfPrevious.index));

        // Update the running paths
        const witness: LowNullifierWitnessData = {
          preimage: lowNullifierPreimage,
          index: BigInt(indexOfPrevious.index),
          siblingPath: siblingPath,
        };
        lowNullifierWitnesses.push(witness);

        // The low nullifier the inserted value will have
        const currentLeafLowNullifier = new NullifierLeafPreimage(
          new Fr(newValue),
          new Fr(lowNullifier.nextValue),
          Number(lowNullifier.nextIndex),
        );
        pendingInsertionSubtree.push(currentLeafLowNullifier);

        // Update the old low nullifier
        lowNullifier.nextValue = newValue;
        lowNullifier.nextIndex = startInsertionIndex + BigInt(i);

        await this.db.updateLeaf(MerkleTreeId.NULLIFIER_TREE, lowNullifier, BigInt(indexOfPrevious.index));
      }
    }

    const newNullifiersSubtreeSiblingPath = await this.getSubtreeSiblingPath(
      MerkleTreeId.NULLIFIER_TREE,
      BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
    );

    // Perform batch insertion of new pending values
    for (let i = 0; i < pendingInsertionSubtree.length; i++) {
      const asLeafData: LeafData = {
        value: pendingInsertionSubtree[i].leafValue.value,
        nextValue: pendingInsertionSubtree[i].nextValue.value,
        nextIndex: BigInt(pendingInsertionSubtree[i].nextIndex),
      };

      await this.db.updateLeaf(MerkleTreeId.NULLIFIER_TREE, asLeafData, startInsertionIndex + BigInt(i));
    }

    return [lowNullifierWitnesses, newNullifiersSubtreeSiblingPath];
  }

  protected async processPublicDataUpdateRequests(tx: ProcessedTx) {
    const newPublicDataUpdateRequestsSiblingPaths: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>[] = [];
    for (const publicDataUpdateRequest of tx.data.end.publicDataUpdateRequests) {
      const index = publicDataUpdateRequest.leafIndex.value;
      const path = await this.db.getSiblingPath(MerkleTreeId.PUBLIC_DATA_TREE, index);
      await this.db.updateLeaf(MerkleTreeId.PUBLIC_DATA_TREE, publicDataUpdateRequest.newValue.toBuffer(), index);
      const witness = new MembershipWitness(PUBLIC_DATA_TREE_HEIGHT, index, path.data.map(Fr.fromBuffer));
      newPublicDataUpdateRequestsSiblingPaths.push(witness);
    }
    return newPublicDataUpdateRequestsSiblingPaths;
  }

  protected async getPublicDataReadsSiblingPaths(tx: ProcessedTx) {
    const newPublicDataReadsSiblingPaths: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>[] = [];
    for (const publicDataRead of tx.data.end.publicDataReads) {
      const index = publicDataRead.leafIndex.value;
      const path = await this.db.getSiblingPath(MerkleTreeId.PUBLIC_DATA_TREE, index);
      const witness = new MembershipWitness(PUBLIC_DATA_TREE_HEIGHT, index, path.data.map(Fr.fromBuffer));
      newPublicDataReadsSiblingPaths.push(witness);
    }
    return newPublicDataReadsSiblingPaths;
  }

  // Builds the base rollup inputs, updating the contract, nullifier, and data trees in the process
  protected async buildBaseRollupInput(left: ProcessedTx, right: ProcessedTx) {
    const wasm = await CircuitsWasm.get();

    // Get trees info before any changes hit
    const constants = await this.getConstantBaseRollupData();
    const startNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const startContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const startPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    const startPublicDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE);

    // Get the subtree sibling paths for the circuit
    const newCommitmentsSubtreeSiblingPath = await this.getSubtreeSiblingPath(
      MerkleTreeId.PRIVATE_DATA_TREE,
      BaseRollupInputs.PRIVATE_DATA_SUBTREE_HEIGHT,
    );
    const newContractsSubtreeSiblingPath = await this.getSubtreeSiblingPath(
      MerkleTreeId.CONTRACT_TREE,
      BaseRollupInputs.CONTRACT_SUBTREE_HEIGHT,
    );

    // Update the contract and private data trees with the new items being inserted to get the new roots
    // that will be used by the next iteration of the base rollup circuit, skipping the empty ones
    const newContracts = flatMap([left, right], tx =>
      tx.data.end.newContracts.map(cd => computeContractLeaf(wasm, cd)),
    );
    const newCommitments = flatMap([left, right], tx => tx.data.end.newCommitments.map(x => x.toBuffer()));
    await this.db.appendLeaves(
      MerkleTreeId.CONTRACT_TREE,
      newContracts.map(x => x.toBuffer()),
    );

    await this.db.appendLeaves(MerkleTreeId.PRIVATE_DATA_TREE, newCommitments);

    // Update the public data tree and get membership witnesses.
    // All public data reads are checked against the unmodified data root when the corresponding tx started,
    // so it's the unmodified tree for tx1, and the one after applying tx1 update request for tx2.
    // Update requests are checked against the tree as it is iteratively updated.
    // See https://github.com/AztecProtocol/aztec3-packages/issues/270#issuecomment-1522258200
    const leftPublicDataReadSiblingPaths = await this.getPublicDataReadsSiblingPaths(left);
    const leftPublicDataUpdateRequestsSiblingPaths = await this.processPublicDataUpdateRequests(left);
    const rightPublicDataReadSiblingPaths = await this.getPublicDataReadsSiblingPaths(right);
    const rightPublicDataUpdateRequestsSiblingPaths = await this.processPublicDataUpdateRequests(right);

    const newPublicDataReadsSiblingPaths = [...leftPublicDataReadSiblingPaths, ...rightPublicDataReadSiblingPaths];
    const newPublicDataUpdateRequestsSiblingPaths = [
      ...leftPublicDataUpdateRequestsSiblingPaths,
      ...rightPublicDataUpdateRequestsSiblingPaths,
    ];

    // Update the nullifier tree, capturing the low nullifier info for each individual operation
    const newNullifiers = [...left.data.end.newNullifiers, ...right.data.end.newNullifiers];

    const [nullifierWitnesses, newNullifiersSubtreeSiblingPath] = await this.performBaseRollupBatchInsertionProofs(
      newNullifiers.map(fr => fr.toBuffer()),
    );
    if (nullifierWitnesses === undefined) {
      throw new Error(`Could not craft nullifier batch insertion proofs`);
    }

    // Extract witness objects from returned data
    const lowNullifierMembershipWitnesses = nullifierWitnesses.map(w =>
      MembershipWitness.fromBufferArray(w.index, w.siblingPath.data),
    );

    return BaseRollupInputs.from({
      constants,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startPrivateDataTreeSnapshot,
      startPublicDataTreeRoot: startPublicDataTreeSnapshot.root,
      newCommitmentsSubtreeSiblingPath,
      newContractsSubtreeSiblingPath,
      newNullifiersSubtreeSiblingPath,
      newPublicDataUpdateRequestsSiblingPaths,
      newPublicDataReadsSiblingPaths,
      lowNullifierLeafPreimages: nullifierWitnesses.map((w: LowNullifierWitnessData) => w.preimage),
      lowNullifierMembershipWitness: lowNullifierMembershipWitnesses,
      kernelData: [this.getKernelDataFor(left), this.getKernelDataFor(right)],
      historicContractsTreeRootMembershipWitnesses: [
        await this.getContractMembershipWitnessFor(left),
        await this.getContractMembershipWitnessFor(right),
      ],
      historicPrivateDataTreeRootMembershipWitnesses: [
        await this.getDataMembershipWitnessFor(left),
        await this.getDataMembershipWitnessFor(right),
      ],
      historicL1ToL2MsgTreeRootMembershipWitnesses: [
        await this.getL1ToL2MessageMembershipWitnessFor(left),
        await this.getL1ToL2MessageMembershipWitnessFor(right),
      ],
    });
  }

  protected makeEmptyMembershipWitness<N extends number>(height: N) {
    return new MembershipWitness(
      height,
      0n,
      times(height, () => new Fr(0n)),
    );
  }
}
