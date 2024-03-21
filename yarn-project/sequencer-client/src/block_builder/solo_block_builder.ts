import { Body, L2Block, MerkleTreeId, TxEffect } from '@aztec/circuit-types';
import { CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  ARCHIVE_HEIGHT,
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  BaseRollupInputs,
  ConstantRollupData,
  GlobalVariables,
  L1_TO_L2_MSG_SUBTREE_HEIGHT,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MembershipWitness,
  MergeRollupInputs,
  NOTE_HASH_SUBTREE_HEIGHT,
  NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_SUBTREE_HEIGHT,
  NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  NullifierLeafPreimage,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  PartialStateReference,
  PreviousRollupData,
  Proof,
  PublicDataTreeLeaf,
  PublicDataTreeLeafPreimage,
  ROLLUP_VK_TREE_HEIGHT,
  RollupKernelCircuitPublicInputs,
  RollupKernelData,
  RollupTypes,
  RootParityInput,
  RootParityInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  StateDiffHints,
  StateReference,
  VK_TREE_HEIGHT,
  VerificationKey,
} from '@aztec/circuits.js';
import { assertPermutation, makeTuple } from '@aztec/foundation/array';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { Tuple, assertLength, toFriendlyJSON } from '@aztec/foundation/serialize';
import { elapsed } from '@aztec/foundation/timer';
import { MerkleTreeOperations } from '@aztec/world-state';

import chunk from 'lodash.chunk';
import { inspect } from 'util';

import { VerificationKeys } from '../mocks/verification_keys.js';
import { RollupProver } from '../prover/index.js';
import { ProcessedTx, toTxEffect } from '../sequencer/processed_tx.js';
import { RollupSimulator } from '../simulator/index.js';
import { BlockBuilder } from './index.js';
import { TreeNames } from './types.js';

const frToBigInt = (fr: Fr) => toBigIntBE(fr.toBuffer());

// Denotes fields that are not used now, but will be in the future
const FUTURE_FR = new Fr(0n);
const FUTURE_NUM = 0;

// Denotes fields that should be deleted
const DELETE_FR = new Fr(0n);

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
    protected debug = createDebugLogger('aztec:sequencer:solo-block-builder'),
  ) {}

  /**
   * Builds an L2 block with the given number containing the given txs, updating state trees.
   * @param globalVariables - Global variables to be used in the block.
   * @param txs - Processed transactions to include in the block.
   * @param l1ToL2Messages - L1 to L2 messages to be part of the block.
   * @param timestamp - Timestamp of the block.
   * @returns The new L2 block and a correctness proof as returned by the root rollup circuit.
   */
  public async buildL2Block(
    globalVariables: GlobalVariables,
    txs: ProcessedTx[],
    l1ToL2Messages: Fr[],
  ): Promise<[L2Block, Proof]> {
    // Check txs are good for processing by checking if all the tree snapshots in header are non-empty
    this.validateTxs(txs);

    // We fill the tx batch with empty txs, we process only one tx at a time for now
    const [circuitsOutput, proof] = await this.runCircuits(globalVariables, txs, l1ToL2Messages);

    // Collect all new nullifiers, commitments, and contracts from all txs in this block
    const txEffects: TxEffect[] = txs.map(tx => toTxEffect(tx));

    const blockBody = new Body(txEffects);

    const l2Block = L2Block.fromFields({
      archive: circuitsOutput.archive,
      header: circuitsOutput.header,
      body: blockBody,
    });

    if (!l2Block.body.getTxsEffectsHash().equals(circuitsOutput.header.contentCommitment.txsEffectsHash)) {
      this.debug(inspect(blockBody));
      throw new Error(
        `Txs effects hash mismatch, ${l2Block.body
          .getTxsEffectsHash()
          .toString('hex')} == ${circuitsOutput.header.contentCommitment.txsEffectsHash.toString('hex')} `,
      );
    }

    return [l2Block, proof];
  }

  protected validateTxs(txs: ProcessedTx[]) {
    for (const tx of txs) {
      const txHeader = tx.data.constants.historicalHeader;
      if (txHeader.state.l1ToL2MessageTree.isZero()) {
        throw new Error(`Empty L1 to L2 messages tree in tx: ${toFriendlyJSON(tx)}`);
      }
      if (txHeader.state.partial.noteHashTree.isZero()) {
        throw new Error(`Empty note hash tree in tx: ${toFriendlyJSON(tx)}`);
      }
      if (txHeader.state.partial.nullifierTree.isZero()) {
        throw new Error(`Empty nullifier tree in tx: ${toFriendlyJSON(tx)}`);
      }
      if (txHeader.state.partial.publicDataTree.isZero()) {
        throw new Error(`Empty public data tree in tx: ${toFriendlyJSON(tx)}`);
      }
    }
  }

  protected async getTreeSnapshot(id: MerkleTreeId): Promise<AppendOnlyTreeSnapshot> {
    const treeInfo = await this.db.getTreeInfo(id);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  }

  protected async runCircuits(
    globalVariables: GlobalVariables,
    txs: ProcessedTx[],
    l1ToL2Messages: Fr[],
  ): Promise<[RootRollupPublicInputs, Proof]> {
    // TODO(#5357): Instead of performing the check bellow pad the txs here.
    // Check that the length of the array of txs is a power of two
    // See https://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
    if (txs.length < 2 || (txs.length & (txs.length - 1)) !== 0) {
      throw new Error(`Length of txs for the block should be a power of two and at least two (got ${txs.length})`);
    }

    // We pad the messages as the circuits expect that.
    const l1ToL2MessagesPadded = padArrayEnd(l1ToL2Messages, Fr.ZERO, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);

    // BASE PARITY CIRCUIT (run in parallel)
    // Note: In the future we will want to cache the results of empty base and root parity circuits so that we don't
    // have to run them. (It will most likely be quite common that some base parity circuits will be "empty")
    let baseParityInputs: BaseParityInputs[] = [];
    let elapsedBaseParityOutputsPromise: Promise<[number, RootParityInput[]]>;
    {
      baseParityInputs = Array.from({ length: NUM_BASE_PARITY_PER_ROOT_PARITY }, (_, i) =>
        BaseParityInputs.fromSlice(l1ToL2MessagesPadded, i),
      );

      const baseParityOutputs: Promise<RootParityInput>[] = [];
      for (const inputs of baseParityInputs) {
        baseParityOutputs.push(this.baseParityCircuit(inputs));
      }
      elapsedBaseParityOutputsPromise = elapsed(() => Promise.all(baseParityOutputs));
    }

    // BASE ROLLUP CIRCUIT (run in parallel)
    let elapsedBaseRollupOutputsPromise: Promise<[number, [BaseOrMergeRollupPublicInputs, Proof][]]>;
    const baseRollupInputs: BaseRollupInputs[] = [];
    {
      // Perform all tree insertions and retrieve snapshots for all base rollups
      const treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>[] = [];
      for (const tx of txs) {
        const input = await this.buildBaseRollupInput(tx, globalVariables);
        baseRollupInputs.push(input);
        const promises = [MerkleTreeId.NOTE_HASH_TREE, MerkleTreeId.NULLIFIER_TREE, MerkleTreeId.PUBLIC_DATA_TREE].map(
          async (id: MerkleTreeId) => {
            return { key: id, value: await this.getTreeSnapshot(id) };
          },
        );
        const snapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot> = new Map(
          (await Promise.all(promises)).map(obj => [obj.key, obj.value]),
        );
        treeSnapshots.push(snapshots);
      }

      // Run the base rollup circuits for the txs in parallel
      const baseRollupOutputs: Promise<[BaseOrMergeRollupPublicInputs, Proof]>[] = [];
      for (let i = 0; i < txs.length; i++) {
        baseRollupOutputs.push(this.baseRollupCircuit(txs[i], baseRollupInputs[i], treeSnapshots[i]));
      }

      elapsedBaseRollupOutputsPromise = elapsed(() => Promise.all(baseRollupOutputs));
    }

    // ROOT PARITY CIRCUIT
    let elapsedRootParityOutputPromise: Promise<[number, RootParityInput]>;
    let rootParityInputs: RootParityInputs;
    {
      // First we await the base parity outputs
      const [duration, baseParityOutputs] = await elapsedBaseParityOutputsPromise;

      // We emit stats for base parity circuits
      for (let i = 0; i < baseParityOutputs.length; i++) {
        this.debug(`Simulated base parity circuit`, {
          eventName: 'circuit-simulation',
          circuitName: 'base-parity',
          duration: duration / baseParityOutputs.length,
          inputSize: baseParityInputs[i].toBuffer().length,
          outputSize: baseParityOutputs[i].toBuffer().length,
        } satisfies CircuitSimulationStats);
      }

      rootParityInputs = new RootParityInputs(
        baseParityOutputs as Tuple<RootParityInput, typeof NUM_BASE_PARITY_PER_ROOT_PARITY>,
      );
      elapsedRootParityOutputPromise = elapsed(() => this.rootParityCircuit(rootParityInputs));
    }

    // MERGE ROLLUP CIRCUIT (each layer run in parallel)
    let mergeOutputLeft: [BaseOrMergeRollupPublicInputs, Proof];
    let mergeOutputRight: [BaseOrMergeRollupPublicInputs, Proof];
    {
      // Run merge rollups in layers until we have only two outputs
      const [duration, mergeInputs] = await elapsedBaseRollupOutputsPromise;

      // We emit stats for base rollup circuits
      for (let i = 0; i < mergeInputs.length; i++) {
        this.debug(`Simulated base rollup circuit`, {
          eventName: 'circuit-simulation',
          circuitName: 'base-rollup',
          duration: duration / mergeInputs.length,
          inputSize: baseRollupInputs[i].toBuffer().length,
          outputSize: mergeInputs[i][0].toBuffer().length,
        } satisfies CircuitSimulationStats);
      }

      let mergeRollupInputs: [BaseOrMergeRollupPublicInputs, Proof][] = mergeInputs;
      while (mergeRollupInputs.length > 2) {
        const mergeInputStructs: MergeRollupInputs[] = [];
        for (const pair of chunk(mergeRollupInputs, 2)) {
          const [r1, r2] = pair;
          mergeInputStructs.push(this.createMergeRollupInputs(r1, r2));
        }

        const [duration, mergeOutputs] = await elapsed(() =>
          Promise.all(mergeInputStructs.map(async input => await this.mergeRollupCircuit(input))),
        );

        // We emit stats for merge rollup circuits
        for (let i = 0; i < mergeOutputs.length; i++) {
          this.debug(`Simulated merge rollup circuit`, {
            eventName: 'circuit-simulation',
            circuitName: 'merge-rollup',
            duration: duration / mergeOutputs.length,
            inputSize: mergeInputStructs[i].toBuffer().length,
            outputSize: mergeOutputs[i][0].toBuffer().length,
          } satisfies CircuitSimulationStats);
        }
        mergeRollupInputs = mergeOutputs;
      }

      // Run the root rollup with the last two merge rollups (or base, if no merge layers)
      [mergeOutputLeft, mergeOutputRight] = mergeRollupInputs;
    }

    // Finally, we emit stats for root parity circuit
    const [duration, rootParityOutput] = await elapsedRootParityOutputPromise;
    this.debug(`Simulated root parity circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'root-parity',
      duration: duration,
      inputSize: rootParityInputs.toBuffer().length,
      outputSize: rootParityOutput.toBuffer().length,
    } satisfies CircuitSimulationStats);

    return this.rootRollupCircuit(mergeOutputLeft, mergeOutputRight, rootParityOutput, l1ToL2MessagesPadded);
  }

  protected async baseParityCircuit(inputs: BaseParityInputs): Promise<RootParityInput> {
    this.debug(`Running base parity circuit`);
    const parityPublicInputs = await this.simulator.baseParityCircuit(inputs);
    const proof = await this.prover.getBaseParityProof(inputs, parityPublicInputs);
    return new RootParityInput(proof, parityPublicInputs);
  }

  protected async rootParityCircuit(inputs: RootParityInputs): Promise<RootParityInput> {
    this.debug(`Running root parity circuit`);
    const parityPublicInputs = await this.simulator.rootParityCircuit(inputs);
    const proof = await this.prover.getRootParityProof(inputs, parityPublicInputs);
    return new RootParityInput(proof, parityPublicInputs);
  }

  protected async baseRollupCircuit(
    tx: ProcessedTx,
    inputs: BaseRollupInputs,
    treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>,
  ): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    this.debug(`Running base rollup for ${tx.hash}`);
    const rollupOutput = await this.simulator.baseRollupCircuit(inputs);
    this.validatePartialState(rollupOutput.end, treeSnapshots);
    const proof = await this.prover.getBaseRollupProof(inputs, rollupOutput);
    return [rollupOutput, proof];
  }

  protected createMergeRollupInputs(
    left: [BaseOrMergeRollupPublicInputs, Proof],
    right: [BaseOrMergeRollupPublicInputs, Proof],
  ) {
    const vk = this.getVerificationKey(left[0].rollupType);
    const mergeInputs = new MergeRollupInputs([
      this.getPreviousRollupDataFromPublicInputs(left[0], left[1], vk),
      this.getPreviousRollupDataFromPublicInputs(right[0], right[1], vk),
    ]);
    return mergeInputs;
  }

  protected async mergeRollupCircuit(mergeInputs: MergeRollupInputs): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
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
    l1ToL2Roots: RootParityInput,
    l1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
  ): Promise<[RootRollupPublicInputs, Proof]> {
    this.debug(`Running root rollup circuit`);
    const rootInput = await this.getRootRollupInput(...left, ...right, l1ToL2Roots, l1ToL2Messages);

    // Update the local trees to include the l1 to l2 messages
    await this.db.appendLeaves(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      l1ToL2Messages.map(m => m.toBuffer()),
    );

    // Simulate and get proof for the root circuit
    const rootOutput = await this.simulator.rootRollupCircuit(rootInput);

    const rootProof = await this.prover.getRootRollupProof(rootInput, rootOutput);

    this.debug(`Updating archive with new header`);
    await this.db.updateArchive(rootOutput.header);

    await this.validateRootOutput(rootOutput);

    return [rootOutput, rootProof];
  }

  protected validatePartialState(
    partialState: PartialStateReference,
    treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>,
  ) {
    this.validateSimulatedTree(
      treeSnapshots.get(MerkleTreeId.NOTE_HASH_TREE)!,
      partialState.noteHashTree,
      'NoteHashTree',
    );
    this.validateSimulatedTree(
      treeSnapshots.get(MerkleTreeId.NULLIFIER_TREE)!,
      partialState.nullifierTree,
      'NullifierTree',
    );
    this.validateSimulatedTree(
      treeSnapshots.get(MerkleTreeId.PUBLIC_DATA_TREE)!,
      partialState.publicDataTree,
      'PublicDataTree',
    );
  }

  protected async validateState(state: StateReference) {
    const promises = [MerkleTreeId.NOTE_HASH_TREE, MerkleTreeId.NULLIFIER_TREE, MerkleTreeId.PUBLIC_DATA_TREE].map(
      async (id: MerkleTreeId) => {
        return { key: id, value: await this.getTreeSnapshot(id) };
      },
    );
    const snapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot> = new Map(
      (await Promise.all(promises)).map(obj => [obj.key, obj.value]),
    );
    this.validatePartialState(state.partial, snapshots);
    this.validateSimulatedTree(
      await this.getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE),
      state.l1ToL2MessageTree,
      'L1ToL2MessageTree',
    );
  }

  // Validate that the roots of all local trees match the output of the root circuit simulation
  protected async validateRootOutput(rootOutput: RootRollupPublicInputs) {
    await Promise.all([
      this.validateState(rootOutput.header.state),
      this.validateSimulatedTree(await this.getTreeSnapshot(MerkleTreeId.ARCHIVE), rootOutput.archive, 'Archive'),
    ]);
  }

  // Helper for comparing two trees snapshots
  protected validateSimulatedTree(
    localTree: AppendOnlyTreeSnapshot,
    simulatedTree: AppendOnlyTreeSnapshot,
    name: TreeNames,
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
    l1ToL2Roots: RootParityInput,
    newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
  ) {
    const vk = this.getVerificationKey(rollupOutputLeft.rollupType);
    const previousRollupData: RootRollupInputs['previousRollupData'] = [
      this.getPreviousRollupDataFromPublicInputs(rollupOutputLeft, rollupProofLeft, vk),
      this.getPreviousRollupDataFromPublicInputs(rollupOutputRight, rollupProofRight, vk),
    ];

    const getRootTreeSiblingPath = async (treeId: MerkleTreeId) => {
      const { size } = await this.db.getTreeInfo(treeId);
      const path = await this.db.getSiblingPath(treeId, size);
      return path.toFields();
    };

    const newL1ToL2MessageTreeRootSiblingPathArray = await this.getSubtreeSiblingPath(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      L1_TO_L2_MSG_SUBTREE_HEIGHT,
    );

    const newL1ToL2MessageTreeRootSiblingPath = makeTuple(
      L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
      i =>
        i < newL1ToL2MessageTreeRootSiblingPathArray.length ? newL1ToL2MessageTreeRootSiblingPathArray[i] : Fr.ZERO,
      0,
    );

    // Get tree snapshots
    const startL1ToL2MessageTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE);

    // Get blocks tree
    const startArchiveSnapshot = await this.getTreeSnapshot(MerkleTreeId.ARCHIVE);
    const newArchiveSiblingPathArray = await getRootTreeSiblingPath(MerkleTreeId.ARCHIVE);

    const newArchiveSiblingPath = makeTuple(
      ARCHIVE_HEIGHT,
      i => (i < newArchiveSiblingPathArray.length ? newArchiveSiblingPathArray[i] : Fr.ZERO),
      0,
    );

    return RootRollupInputs.from({
      previousRollupData,
      l1ToL2Roots,
      newL1ToL2Messages,
      newL1ToL2MessageTreeRootSiblingPath,
      startL1ToL2MessageTreeSnapshot,
      startArchiveSnapshot,
      newArchiveSiblingPath,
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
      new MembershipWitness(
        ROLLUP_VK_TREE_HEIGHT,
        BigInt(FUTURE_NUM),
        makeTuple(ROLLUP_VK_TREE_HEIGHT, () => FUTURE_FR),
      ),
    );
  }

  protected getKernelDataFor(tx: ProcessedTx): RollupKernelData {
    const inputs = new RollupKernelCircuitPublicInputs(
      tx.data.aggregationObject,
      tx.data.combinedData,
      tx.data.constants,
    );
    return new RollupKernelData(
      inputs,
      tx.proof,

      // VK for the kernel circuit
      this.vks.privateKernelCircuit,

      // MembershipWitness for a VK tree to be implemented in the future
      FUTURE_NUM,
      assertLength(Array(VK_TREE_HEIGHT).fill(FUTURE_FR), VK_TREE_HEIGHT),
    );
  }

  // Scan a tree searching for a specific value and return a membership witness proof for it
  protected async getMembershipWitnessFor<N extends number>(
    value: Fr,
    treeId: MerkleTreeId,
    height: N,
  ): Promise<MembershipWitness<N>> {
    // If this is an empty tx, then just return zeroes
    if (value.isZero()) {
      return this.makeEmptyMembershipWitness(height);
    }

    const index = await this.db.findLeafIndex(treeId, value.toBuffer());
    if (index === undefined) {
      throw new Error(`Leaf with value ${value} not found in tree ${MerkleTreeId[treeId]}`);
    }
    const path = await this.db.getSiblingPath(treeId, index);
    return new MembershipWitness(height, index, assertLength(path.toFields(), height));
  }

  protected async getConstantRollupData(globalVariables: GlobalVariables): Promise<ConstantRollupData> {
    return ConstantRollupData.from({
      baseRollupVkHash: DELETE_FR,
      mergeRollupVkHash: DELETE_FR,
      privateKernelVkTreeRoot: FUTURE_FR,
      publicKernelVkTreeRoot: FUTURE_FR,
      lastArchive: await this.getTreeSnapshot(MerkleTreeId.ARCHIVE),
      globalVariables,
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
    if (!prevValueIndex) {
      throw new Error(`Nullifier tree should have one initial leaf`);
    }
    const prevValuePreimage = (await this.db.getLeafPreimage(tree, prevValueIndex.index))!;

    const prevValueSiblingPath = await this.db.getSiblingPath(tree, BigInt(prevValueIndex.index));

    return {
      index: prevValueIndex,
      leafPreimage: prevValuePreimage,
      witness: new MembershipWitness(
        NULLIFIER_TREE_HEIGHT,
        BigInt(prevValueIndex.index),
        assertLength(prevValueSiblingPath.toFields(), NULLIFIER_TREE_HEIGHT),
      ),
    };
  }

  protected async getSubtreeSiblingPath(treeId: MerkleTreeId, subtreeHeight: number): Promise<Fr[]> {
    const nextAvailableLeafIndex = await this.db.getTreeInfo(treeId).then(t => t.size);
    const fullSiblingPath = await this.db.getSiblingPath(treeId, nextAvailableLeafIndex);

    // Drop the first subtreeHeight items since we only care about the path to the subtree root
    return fullSiblingPath.getSubtreeSiblingPath(subtreeHeight).toFields();
  }

  protected async processPublicDataUpdateRequests(tx: ProcessedTx) {
    const combinedPublicDataUpdateRequests = tx.data.combinedData.publicDataUpdateRequests.map(updateRequest => {
      return new PublicDataTreeLeaf(updateRequest.leafSlot, updateRequest.newValue);
    });
    const { lowLeavesWitnessData, newSubtreeSiblingPath, sortedNewLeaves, sortedNewLeavesIndexes } =
      await this.db.batchInsert(
        MerkleTreeId.PUBLIC_DATA_TREE,
        combinedPublicDataUpdateRequests.map(x => x.toBuffer()),
        // TODO(#3675) remove oldValue from update requests
        PUBLIC_DATA_SUBTREE_HEIGHT,
      );

    if (lowLeavesWitnessData === undefined) {
      throw new Error(`Could not craft public data batch insertion proofs`);
    }

    const sortedPublicDataWrites = makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
      return PublicDataTreeLeaf.fromBuffer(sortedNewLeaves[i]);
    });

    const sortedPublicDataWritesIndexes = makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
      return sortedNewLeavesIndexes[i];
    });

    const subtreeSiblingPathAsFields = newSubtreeSiblingPath.toFields();
    const newPublicDataSubtreeSiblingPath = makeTuple(PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, i => {
      return subtreeSiblingPathAsFields[i];
    });

    const lowPublicDataWritesMembershipWitnesses: Tuple<
      MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    > = makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
      const witness = lowLeavesWitnessData[i];
      return MembershipWitness.fromBufferArray(
        witness.index,
        assertLength(witness.siblingPath.toBufferArray(), PUBLIC_DATA_TREE_HEIGHT),
      );
    });

    const lowPublicDataWritesPreimages: Tuple<
      PublicDataTreeLeafPreimage,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    > = makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
      return lowLeavesWitnessData[i].leafPreimage as PublicDataTreeLeafPreimage;
    });

    // validate that the sortedPublicDataWrites and sortedPublicDataWritesIndexes are in the correct order
    // otherwise it will just fail in the circuit
    assertPermutation(combinedPublicDataUpdateRequests, sortedPublicDataWrites, sortedPublicDataWritesIndexes, (a, b) =>
      a.equals(b),
    );

    return {
      lowPublicDataWritesPreimages,
      lowPublicDataWritesMembershipWitnesses,
      newPublicDataSubtreeSiblingPath,
      sortedPublicDataWrites,
      sortedPublicDataWritesIndexes,
    };
  }

  protected async getPublicDataReadsInfo(tx: ProcessedTx) {
    const newPublicDataReadsWitnesses: Tuple<
      MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_PUBLIC_DATA_READS_PER_TX
    > = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () => MembershipWitness.empty(PUBLIC_DATA_TREE_HEIGHT, 0n));

    const newPublicDataReadsPreimages: Tuple<PublicDataTreeLeafPreimage, typeof MAX_PUBLIC_DATA_READS_PER_TX> =
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () => PublicDataTreeLeafPreimage.empty());

    for (const i in tx.data.validationRequests.publicDataReads) {
      const leafSlot = tx.data.validationRequests.publicDataReads[i].leafSlot.value;
      const lowLeafResult = await this.db.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot);
      if (!lowLeafResult) {
        throw new Error(`Public data tree should have one initial leaf`);
      }
      const preimage = await this.db.getLeafPreimage(MerkleTreeId.PUBLIC_DATA_TREE, lowLeafResult.index);
      const path = await this.db.getSiblingPath(MerkleTreeId.PUBLIC_DATA_TREE, lowLeafResult.index);
      newPublicDataReadsWitnesses[i] = new MembershipWitness(
        PUBLIC_DATA_TREE_HEIGHT,
        BigInt(lowLeafResult.index),
        path.toTuple<typeof PUBLIC_DATA_TREE_HEIGHT>(),
      );
      newPublicDataReadsPreimages[i] = preimage! as PublicDataTreeLeafPreimage;
    }
    return {
      newPublicDataReadsWitnesses,
      newPublicDataReadsPreimages,
    };
  }

  // Builds the base rollup inputs, updating the contract, nullifier, and data trees in the process
  protected async buildBaseRollupInput(tx: ProcessedTx, globalVariables: GlobalVariables) {
    // Get trees info before any changes hit
    const constants = await this.getConstantRollupData(globalVariables);
    const start = new PartialStateReference(
      await this.getTreeSnapshot(MerkleTreeId.NOTE_HASH_TREE),
      await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE),
      await this.getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE),
    );

    // Get the subtree sibling paths for the circuit
    const noteHashSubtreeSiblingPathArray = await this.getSubtreeSiblingPath(
      MerkleTreeId.NOTE_HASH_TREE,
      NOTE_HASH_SUBTREE_HEIGHT,
    );

    const noteHashSubtreeSiblingPath = makeTuple(NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH, i =>
      i < noteHashSubtreeSiblingPathArray.length ? noteHashSubtreeSiblingPathArray[i] : Fr.ZERO,
    );

    // Update the note hash trees with the new items being inserted to get the new roots
    // that will be used by the next iteration of the base rollup circuit, skipping the empty ones
    const newNoteHashes = tx.data.combinedData.newNoteHashes.map(x => x.value.toBuffer());
    await this.db.appendLeaves(MerkleTreeId.NOTE_HASH_TREE, newNoteHashes);

    // The read witnesses for a given TX should be generated before the writes of the same TX are applied.
    // All reads that refer to writes in the same tx are transient and can be simplified out.
    const txPublicDataReadsInfo = await this.getPublicDataReadsInfo(tx);
    const txPublicDataUpdateRequestInfo = await this.processPublicDataUpdateRequests(tx);

    // Update the nullifier tree, capturing the low nullifier info for each individual operation
    const {
      lowLeavesWitnessData: nullifierWitnessLeaves,
      newSubtreeSiblingPath: newNullifiersSubtreeSiblingPath,
      sortedNewLeaves: sortedNewNullifiers,
      sortedNewLeavesIndexes,
    } = await this.db.batchInsert(
      MerkleTreeId.NULLIFIER_TREE,
      tx.data.combinedData.newNullifiers.map(sideEffectLinkedToNoteHash => sideEffectLinkedToNoteHash.value.toBuffer()),
      NULLIFIER_SUBTREE_HEIGHT,
    );
    if (nullifierWitnessLeaves === undefined) {
      throw new Error(`Could not craft nullifier batch insertion proofs`);
    }

    // Extract witness objects from returned data
    const nullifierPredecessorMembershipWitnessesWithoutPadding: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>[] =
      nullifierWitnessLeaves.map(l =>
        MembershipWitness.fromBufferArray(l.index, assertLength(l.siblingPath.toBufferArray(), NULLIFIER_TREE_HEIGHT)),
      );

    const nullifierSubtreeSiblingPathArray = newNullifiersSubtreeSiblingPath.toFields();

    const nullifierSubtreeSiblingPath = makeTuple(NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, i =>
      i < nullifierSubtreeSiblingPathArray.length ? nullifierSubtreeSiblingPathArray[i] : Fr.ZERO,
    );

    const publicDataSiblingPath = txPublicDataUpdateRequestInfo.newPublicDataSubtreeSiblingPath;

    const stateDiffHints = StateDiffHints.from({
      nullifierPredecessorPreimages: makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i =>
        i < nullifierWitnessLeaves.length
          ? (nullifierWitnessLeaves[i].leafPreimage as NullifierLeafPreimage)
          : NullifierLeafPreimage.empty(),
      ),
      nullifierPredecessorMembershipWitnesses: makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i =>
        i < nullifierPredecessorMembershipWitnessesWithoutPadding.length
          ? nullifierPredecessorMembershipWitnessesWithoutPadding[i]
          : this.makeEmptyMembershipWitness(NULLIFIER_TREE_HEIGHT),
      ),
      sortedNullifiers: makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i => Fr.fromBuffer(sortedNewNullifiers[i])),
      sortedNullifierIndexes: makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i => sortedNewLeavesIndexes[i]),
      noteHashSubtreeSiblingPath,
      nullifierSubtreeSiblingPath,
      publicDataSiblingPath,
    });

    const blockHash = tx.data.constants.historicalHeader.hash();
    const archiveRootMembershipWitness = await this.getMembershipWitnessFor(
      blockHash,
      MerkleTreeId.ARCHIVE,
      ARCHIVE_HEIGHT,
    );

    return BaseRollupInputs.from({
      kernelData: this.getKernelDataFor(tx),
      start,
      stateDiffHints,

      sortedPublicDataWrites: txPublicDataUpdateRequestInfo.sortedPublicDataWrites,
      sortedPublicDataWritesIndexes: txPublicDataUpdateRequestInfo.sortedPublicDataWritesIndexes,
      lowPublicDataWritesPreimages: txPublicDataUpdateRequestInfo.lowPublicDataWritesPreimages,
      lowPublicDataWritesMembershipWitnesses: txPublicDataUpdateRequestInfo.lowPublicDataWritesMembershipWitnesses,
      publicDataReadsPreimages: txPublicDataReadsInfo.newPublicDataReadsPreimages,
      publicDataReadsMembershipWitnesses: txPublicDataReadsInfo.newPublicDataReadsWitnesses,

      archiveRootMembershipWitness,

      constants,
    });
  }

  protected makeEmptyMembershipWitness<N extends number>(height: N) {
    return new MembershipWitness(
      height,
      0n,
      makeTuple(height, () => Fr.ZERO),
    );
  }
}
