import { ContractData, L2Block, L2BlockL2Logs, MerkleTreeId, PublicDataWrite, TxL2Logs } from '@aztec/circuit-types';
import {
  ARCHIVE_HEIGHT,
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CONTRACT_SUBTREE_HEIGHT,
  CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
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
  NullifierLeafPreimage,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  PartialStateReference,
  PreviousKernelData,
  PreviousRollupData,
  Proof,
  PublicDataTreeLeaf,
  PublicDataTreeLeafPreimage,
  ROLLUP_VK_TREE_HEIGHT,
  RollupTypes,
  RootRollupInputs,
  RootRollupPublicInputs,
  SideEffect,
  SideEffectLinkedToNoteHash,
  StateDiffHints,
  StateReference,
  VK_TREE_HEIGHT,
  VerificationKey,
} from '@aztec/circuits.js';
import {
  computeBlockHash,
  computeBlockHashWithGlobals,
  computeContractLeaf,
  computeGlobalsHash,
} from '@aztec/circuits.js/abis';
import { makeTuple } from '@aztec/foundation/array';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { Tuple, assertLength, toFriendlyJSON } from '@aztec/foundation/serialize';
import { MerkleTreeOperations } from '@aztec/world-state';

import chunk from 'lodash.chunk';

import { VerificationKeys } from '../mocks/verification_keys.js';
import { RollupProver } from '../prover/index.js';
import { ProcessedTx } from '../sequencer/processed_tx.js';
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
   * @param newL1ToL2Messages - L1 to L2 messages to be part of the block.
   * @param timestamp - Timestamp of the block.
   * @returns The new L2 block and a correctness proof as returned by the root rollup circuit.
   */
  public async buildL2Block(
    globalVariables: GlobalVariables,
    txs: ProcessedTx[],
    newL1ToL2Messages: Fr[],
  ): Promise<[L2Block, Proof]> {
    // Check txs are good for processing
    this.validateTxs(txs);

    // We fill the tx batch with empty txs, we process only one tx at a time for now
    const [circuitsOutput, proof] = await this.runCircuits(globalVariables, txs, newL1ToL2Messages);

    // Collect all new nullifiers, commitments, and contracts from all txs in this block
    const newNullifiers = txs.flatMap(tx => tx.data.end.newNullifiers);
    const newCommitments = txs.flatMap(tx => tx.data.end.newCommitments);
    const newContracts = txs.flatMap(tx => tx.data.end.newContracts).map(cd => computeContractLeaf(cd));
    const newContractData = txs
      .flatMap(tx => tx.data.end.newContracts)
      .map(n => new ContractData(n.contractAddress, n.portalContractAddress));
    const newPublicDataWrites = txs.flatMap(tx =>
      tx.data.end.publicDataUpdateRequests.map(t => new PublicDataWrite(t.leafSlot, t.newValue)),
    );
    const newL2ToL1Msgs = txs.flatMap(tx => tx.data.end.newL2ToL1Msgs);

    // Consolidate logs data from all txs
    const encryptedLogsArr: TxL2Logs[] = [];
    const unencryptedLogsArr: TxL2Logs[] = [];
    for (const tx of txs) {
      const encryptedLogs = tx.encryptedLogs || new TxL2Logs([]);
      encryptedLogsArr.push(encryptedLogs);
      const unencryptedLogs = tx.unencryptedLogs || new TxL2Logs([]);
      unencryptedLogsArr.push(unencryptedLogs);
    }
    const newEncryptedLogs = new L2BlockL2Logs(encryptedLogsArr);
    const newUnencryptedLogs = new L2BlockL2Logs(unencryptedLogsArr);

    const l2Block = L2Block.fromFields({
      archive: circuitsOutput.archive,
      header: circuitsOutput.header,
      newCommitments: newCommitments.map((c: SideEffect) => c.value),
      newNullifiers: newNullifiers.map((n: SideEffectLinkedToNoteHash) => n.value),
      newL2ToL1Msgs,
      newContracts,
      newContractData,
      newPublicDataWrites,
      newL1ToL2Messages,
      newEncryptedLogs,
      newUnencryptedLogs,
    });

    if (!l2Block.getCalldataHash().equals(circuitsOutput.header.bodyHash)) {
      throw new Error(
        `Calldata hash mismatch, ${l2Block
          .getCalldataHash()
          .toString('hex')} == ${circuitsOutput.header.bodyHash.toString('hex')} `,
      );
    }

    return [l2Block, proof];
  }

  protected validateTxs(txs: ProcessedTx[]) {
    for (const tx of txs) {
      for (const historicalTreeRoot of [
        'noteHashTreeRoot',
        'contractTreeRoot',
        'nullifierTreeRoot',
        'l1ToL2MessageTreeRoot',
      ] as const) {
        if (tx.data.constants.blockHeader[historicalTreeRoot].isZero()) {
          throw new Error(`Empty ${historicalTreeRoot} for tx: ${toFriendlyJSON(tx)}`);
        }
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
    newL1ToL2Messages: Fr[],
  ): Promise<[RootRollupPublicInputs, Proof]> {
    // Check that the length of the array of txs is a power of two
    // See https://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
    if (txs.length < 2 || (txs.length & (txs.length - 1)) !== 0) {
      throw new Error(`Length of txs for the block should be a power of two and at least two (got ${txs.length})`);
    }

    // padArrayEnd throws if the array is already full. Otherwise it pads till we reach the required size
    const newL1ToL2MessagesTuple = padArrayEnd(newL1ToL2Messages, Fr.ZERO, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);

    // Run the base rollup circuits for the txs
    const baseRollupOutputs: [BaseOrMergeRollupPublicInputs, Proof][] = [];
    for (const tx of txs) {
      baseRollupOutputs.push(await this.baseRollupCircuit(tx, globalVariables));
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
    return this.rootRollupCircuit(mergeOutputLeft, mergeOutputRight, newL1ToL2MessagesTuple);
  }

  protected async baseRollupCircuit(
    tx: ProcessedTx,
    globalVariables: GlobalVariables,
  ): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    this.debug(`Running base rollup for ${tx.hash}`);
    const rollupInput = await this.buildBaseRollupInput(tx, globalVariables);
    const rollupOutput = await this.simulator.baseRollupCircuit(rollupInput);
    await this.validatePartialState(rollupOutput.end);
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
    newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
  ): Promise<[RootRollupPublicInputs, Proof]> {
    this.debug(`Running root rollup circuit`);
    const rootInput = await this.getRootRollupInput(...left, ...right, newL1ToL2Messages);

    // Update the local trees to include the new l1 to l2 messages
    await this.db.appendLeaves(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      newL1ToL2Messages.map(m => m.toBuffer()),
    );

    // Simulate and get proof for the root circuit
    const rootOutput = await this.simulator.rootRollupCircuit(rootInput);

    const rootProof = await this.prover.getRootRollupProof(rootInput, rootOutput);

    // Update the root trees with the latest data and contract tree roots,
    // and validate them against the output of the root circuit simulation
    this.debug(`Updating and validating root trees`);
    const globalVariablesHash = computeGlobalsHash(left[0].constants.globalVariables);
    await this.db.updateLatestGlobalVariablesHash(globalVariablesHash);
    await this.db.updateArchive(globalVariablesHash);

    await this.validateRootOutput(rootOutput);

    return [rootOutput, rootProof];
  }

  async updateArchive(globalVariables: GlobalVariables) {
    // Calculate the block hash and add it to the historical block hashes tree
    const blockHash = await this.calculateBlockHash(globalVariables);
    await this.db.appendLeaves(MerkleTreeId.ARCHIVE, [blockHash.toBuffer()]);
  }

  protected async calculateBlockHash(globals: GlobalVariables) {
    const [noteHashTreeRoot, nullifierTreeRoot, contractTreeRoot, publicDataTreeRoot, l1ToL2MessageTreeRoot] = (
      await Promise.all(
        [
          MerkleTreeId.NOTE_HASH_TREE,
          MerkleTreeId.NULLIFIER_TREE,
          MerkleTreeId.CONTRACT_TREE,
          MerkleTreeId.PUBLIC_DATA_TREE,
          MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
        ].map(tree => this.getTreeSnapshot(tree)),
      )
    ).map(r => r.root);

    const blockHash = computeBlockHashWithGlobals(
      globals,
      noteHashTreeRoot,
      nullifierTreeRoot,
      contractTreeRoot,
      l1ToL2MessageTreeRoot,
      publicDataTreeRoot,
    );
    return blockHash;
  }

  protected async validatePartialState(partialState: PartialStateReference) {
    await Promise.all([
      this.validateSimulatedTree(
        await this.getTreeSnapshot(MerkleTreeId.NOTE_HASH_TREE),
        partialState.noteHashTree,
        'NoteHashTree',
      ),
      this.validateSimulatedTree(
        await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE),
        partialState.nullifierTree,
        'NullifierTree',
      ),
      this.validateSimulatedTree(
        await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE),
        partialState.contractTree,
        'ContractTree',
      ),
      this.validateSimulatedTree(
        await this.getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE),
        partialState.publicDataTree,
        'PublicDataTree',
      ),
    ]);
  }

  protected async validateState(state: StateReference) {
    await Promise.all([
      this.validateSimulatedTree(
        await this.getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE),
        state.l1ToL2MessageTree,
        'L1ToL2MessageTree',
      ),
      this.validatePartialState(state.partial),
    ]);
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
    newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
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
      return path.toFieldArray();
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

  protected getKernelDataFor(tx: ProcessedTx) {
    return new PreviousKernelData(
      tx.data,
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
    return new MembershipWitness(height, index, assertLength(path.toFieldArray(), height));
  }

  protected getHistoricalTreesMembershipWitnessFor(tx: ProcessedTx) {
    const blockHeader = tx.data.constants.blockHeader;
    const { noteHashTreeRoot, nullifierTreeRoot, contractTreeRoot, l1ToL2MessageTreeRoot, publicDataTreeRoot } =
      blockHeader;
    const blockHash = computeBlockHash(
      blockHeader.globalVariablesHash,
      noteHashTreeRoot,
      nullifierTreeRoot,
      contractTreeRoot,
      l1ToL2MessageTreeRoot,
      publicDataTreeRoot,
    );
    return this.getMembershipWitnessFor(blockHash, MerkleTreeId.ARCHIVE, ARCHIVE_HEIGHT);
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
        assertLength(prevValueSiblingPath.toFieldArray(), NULLIFIER_TREE_HEIGHT),
      ),
    };
  }

  protected async getSubtreeSiblingPath(treeId: MerkleTreeId, subtreeHeight: number): Promise<Fr[]> {
    const nextAvailableLeafIndex = await this.db.getTreeInfo(treeId).then(t => t.size);
    const fullSiblingPath = await this.db.getSiblingPath(treeId, nextAvailableLeafIndex);

    // Drop the first subtreeHeight items since we only care about the path to the subtree root
    return fullSiblingPath.getSubtreeSiblingPath(subtreeHeight).toFieldArray();
  }

  protected async processPublicDataUpdateRequests(tx: ProcessedTx) {
    const { lowLeavesWitnessData, newSubtreeSiblingPath, sortedNewLeaves, sortedNewLeavesIndexes } =
      await this.db.batchInsert(
        MerkleTreeId.PUBLIC_DATA_TREE,
        // TODO(#3675) remove oldValue from update requests
        tx.data.end.publicDataUpdateRequests.map(updateRequest => {
          return new PublicDataTreeLeaf(updateRequest.leafSlot, updateRequest.newValue).toBuffer();
        }),
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

    const subtreeSiblingPathAsFields = newSubtreeSiblingPath.toFieldArray();
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
    for (const i in tx.data.end.publicDataReads) {
      const leafSlot = tx.data.end.publicDataReads[i].leafSlot.value;
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
      await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE),
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

    const contractSubtreeSiblingPathArray = await this.getSubtreeSiblingPath(
      MerkleTreeId.CONTRACT_TREE,
      CONTRACT_SUBTREE_HEIGHT,
    );

    const contractSubtreeSiblingPath = makeTuple(CONTRACT_SUBTREE_SIBLING_PATH_LENGTH, i =>
      i < contractSubtreeSiblingPathArray.length ? contractSubtreeSiblingPathArray[i] : Fr.ZERO,
    );

    // Update the contract and note hash trees with the new items being inserted to get the new roots
    // that will be used by the next iteration of the base rollup circuit, skipping the empty ones
    const newContracts = tx.data.end.newContracts.map(cd => computeContractLeaf(cd));
    const newCommitments = tx.data.end.newCommitments.map(x => x.value.toBuffer());
    await this.db.appendLeaves(
      MerkleTreeId.CONTRACT_TREE,
      newContracts.map(x => x.toBuffer()),
    );

    await this.db.appendLeaves(MerkleTreeId.NOTE_HASH_TREE, newCommitments);

    // The read witnesses for a given TX should be generated before the writes of the same TX are applied.
    // All reads that refer to writes in the same tx are transient and can be simplified out.
    const txPublicDataReadsInfo = await this.getPublicDataReadsInfo(tx);
    const txPublicDataUpdateRequestInfo = await this.processPublicDataUpdateRequests(tx);

    // Update the nullifier tree, capturing the low nullifier info for each individual operation
    const newNullifiers = tx.data.end.newNullifiers;

    const {
      lowLeavesWitnessData: nullifierWitnessLeaves,
      newSubtreeSiblingPath: newNullifiersSubtreeSiblingPath,
      sortedNewLeaves: sortedNewNullifiers,
      sortedNewLeavesIndexes,
    } = await this.db.batchInsert(
      MerkleTreeId.NULLIFIER_TREE,
      newNullifiers.map(sideEffectLinkedToNoteHash => sideEffectLinkedToNoteHash.value.toBuffer()),
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

    const nullifierSubtreeSiblingPathArray = newNullifiersSubtreeSiblingPath.toFieldArray();

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
      contractSubtreeSiblingPath,
      publicDataSiblingPath,
    });

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

      archiveRootMembershipWitness: await this.getHistoricalTreesMembershipWitnessFor(tx),

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
