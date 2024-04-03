import { MerkleTreeId, type ProcessedTx } from '@aztec/circuit-types';
import {
  ARCHIVE_HEIGHT,
  AppendOnlyTreeSnapshot,
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  BaseRollupInputs,
  ConstantRollupData,
  Fr,
  type GlobalVariables,
  KernelData,
  L1_TO_L2_MSG_SUBTREE_HEIGHT,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MembershipWitness,
  MergeRollupInputs,
  NOTE_HASH_SUBTREE_HEIGHT,
  NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_SUBTREE_HEIGHT,
  NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  type NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NullifierLeafPreimage,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  PartialStateReference,
  PreviousRollupData,
  type Proof,
  PublicDataTreeLeaf,
  type PublicDataTreeLeafPreimage,
  ROLLUP_VK_TREE_HEIGHT,
  RollupTypes,
  RootParityInput,
  type RootParityInputs,
  RootRollupInputs,
  type RootRollupPublicInputs,
  StateDiffHints,
  type StateReference,
  VK_TREE_HEIGHT,
  type VerificationKey,
} from '@aztec/circuits.js';
import { assertPermutation, makeTuple } from '@aztec/foundation/array';
import { type DebugLogger } from '@aztec/foundation/log';
import { type Tuple, assertLength, toFriendlyJSON } from '@aztec/foundation/serialize';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { type VerificationKeys, getVerificationKeys } from '../mocks/verification_keys.js';
import { type RollupProver } from '../prover/index.js';
import { type RollupSimulator } from '../simulator/rollup.js';

// Denotes fields that are not used now, but will be in the future
const FUTURE_FR = new Fr(0n);
const FUTURE_NUM = 0;

// Denotes fields that should be deleted
const DELETE_FR = new Fr(0n);

/**
 * Type representing the names of the trees for the base rollup.
 */
type BaseTreeNames = 'NoteHashTree' | 'ContractTree' | 'NullifierTree' | 'PublicDataTree';
/**
 * Type representing the names of the trees.
 */
export type TreeNames = BaseTreeNames | 'L1ToL2MessageTree' | 'Archive';

// Builds the base rollup inputs, updating the contract, nullifier, and data trees in the process
export async function buildBaseRollupInput(
  tx: ProcessedTx,
  globalVariables: GlobalVariables,
  db: MerkleTreeOperations,
) {
  // Get trees info before any changes hit
  const constants = await getConstantRollupData(globalVariables, db);
  const start = new PartialStateReference(
    await getTreeSnapshot(MerkleTreeId.NOTE_HASH_TREE, db),
    await getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE, db),
    await getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE, db),
  );
  // Get the subtree sibling paths for the circuit
  const noteHashSubtreeSiblingPathArray = await getSubtreeSiblingPath(
    MerkleTreeId.NOTE_HASH_TREE,
    NOTE_HASH_SUBTREE_HEIGHT,
    db,
  );

  const noteHashSubtreeSiblingPath = makeTuple(NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH, i =>
    i < noteHashSubtreeSiblingPathArray.length ? noteHashSubtreeSiblingPathArray[i] : Fr.ZERO,
  );

  // Update the note hash trees with the new items being inserted to get the new roots
  // that will be used by the next iteration of the base rollup circuit, skipping the empty ones
  const newNoteHashes = tx.data.end.newNoteHashes.map(x => x.value);
  await db.appendLeaves(MerkleTreeId.NOTE_HASH_TREE, newNoteHashes);

  // The read witnesses for a given TX should be generated before the writes of the same TX are applied.
  // All reads that refer to writes in the same tx are transient and can be simplified out.
  const txPublicDataUpdateRequestInfo = await processPublicDataUpdateRequests(tx, db);

  // Update the nullifier tree, capturing the low nullifier info for each individual operation
  const {
    lowLeavesWitnessData: nullifierWitnessLeaves,
    newSubtreeSiblingPath: newNullifiersSubtreeSiblingPath,
    sortedNewLeaves: sortedNewNullifiers,
    sortedNewLeavesIndexes,
  } = await db.batchInsert(
    MerkleTreeId.NULLIFIER_TREE,
    tx.data.end.newNullifiers.map(sideEffectLinkedToNoteHash => sideEffectLinkedToNoteHash.value.toBuffer()),
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
        : makeEmptyMembershipWitness(NULLIFIER_TREE_HEIGHT),
    ),
    sortedNullifiers: makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i => Fr.fromBuffer(sortedNewNullifiers[i])),
    sortedNullifierIndexes: makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i => sortedNewLeavesIndexes[i]),
    noteHashSubtreeSiblingPath,
    nullifierSubtreeSiblingPath,
    publicDataSiblingPath,
  });

  const blockHash = tx.data.constants.historicalHeader.hash();
  const archiveRootMembershipWitness = await getMembershipWitnessFor(
    blockHash,
    MerkleTreeId.ARCHIVE,
    ARCHIVE_HEIGHT,
    db,
  );

  return BaseRollupInputs.from({
    kernelData: getKernelDataFor(tx, getVerificationKeys()),
    start,
    stateDiffHints,

    sortedPublicDataWrites: txPublicDataUpdateRequestInfo.sortedPublicDataWrites,
    sortedPublicDataWritesIndexes: txPublicDataUpdateRequestInfo.sortedPublicDataWritesIndexes,
    lowPublicDataWritesPreimages: txPublicDataUpdateRequestInfo.lowPublicDataWritesPreimages,
    lowPublicDataWritesMembershipWitnesses: txPublicDataUpdateRequestInfo.lowPublicDataWritesMembershipWitnesses,

    archiveRootMembershipWitness,

    constants,
  });
}

export function createMergeRollupInputs(
  left: [BaseOrMergeRollupPublicInputs, Proof],
  right: [BaseOrMergeRollupPublicInputs, Proof],
) {
  const vks = getVerificationKeys();
  const vk = left[0].rollupType === RollupTypes.Base ? vks.baseRollupCircuit : vks.mergeRollupCircuit;
  const mergeInputs = new MergeRollupInputs([
    getPreviousRollupDataFromPublicInputs(left[0], left[1], vk),
    getPreviousRollupDataFromPublicInputs(right[0], right[1], vk),
  ]);
  return mergeInputs;
}

export async function executeMergeRollupCircuit(
  mergeInputs: MergeRollupInputs,
  simulator: RollupSimulator,
  prover: RollupProver,
  logger?: DebugLogger,
): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
  logger?.debug(`Running merge rollup circuit`);
  const output = await simulator.mergeRollupCircuit(mergeInputs);
  const proof = await prover.getMergeRollupProof(mergeInputs, output);
  return [output, proof];
}

export async function executeRootRollupCircuit(
  left: [BaseOrMergeRollupPublicInputs, Proof],
  right: [BaseOrMergeRollupPublicInputs, Proof],
  l1ToL2Roots: RootParityInput,
  newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
  simulator: RollupSimulator,
  prover: RollupProver,
  db: MerkleTreeOperations,
  logger?: DebugLogger,
): Promise<[RootRollupPublicInputs, Proof]> {
  logger?.debug(`Running root rollup circuit`);
  const rootInput = await getRootRollupInput(...left, ...right, l1ToL2Roots, newL1ToL2Messages, db);

  // Update the local trees to include the new l1 to l2 messages
  await db.appendLeaves(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, newL1ToL2Messages);

  // Simulate and get proof for the root circuit
  const rootOutput = await simulator.rootRollupCircuit(rootInput);

  const rootProof = await prover.getRootRollupProof(rootInput, rootOutput);

  //TODO(@PhilWindle) Move this to orchestrator to ensure that we are still on the same block
  // Update the archive with the latest block header
  logger?.debug(`Updating and validating root trees`);
  await db.updateArchive(rootOutput.header);

  await validateRootOutput(rootOutput, db);

  return [rootOutput, rootProof];
}

// Validate that the roots of all local trees match the output of the root circuit simulation
export async function validateRootOutput(rootOutput: RootRollupPublicInputs, db: MerkleTreeOperations) {
  await Promise.all([
    validateState(rootOutput.header.state, db),
    validateSimulatedTree(await getTreeSnapshot(MerkleTreeId.ARCHIVE, db), rootOutput.archive, 'Archive'),
  ]);
}

export async function validateState(state: StateReference, db: MerkleTreeOperations) {
  const promises = [MerkleTreeId.NOTE_HASH_TREE, MerkleTreeId.NULLIFIER_TREE, MerkleTreeId.PUBLIC_DATA_TREE].map(
    async (id: MerkleTreeId) => {
      return { key: id, value: await getTreeSnapshot(id, db) };
    },
  );
  const snapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot> = new Map(
    (await Promise.all(promises)).map(obj => [obj.key, obj.value]),
  );
  validatePartialState(state.partial, snapshots);
  validateSimulatedTree(
    await getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, db),
    state.l1ToL2MessageTree,
    'L1ToL2MessageTree',
  );
}

// Builds the inputs for the root rollup circuit, without making any changes to trees
export async function getRootRollupInput(
  rollupOutputLeft: BaseOrMergeRollupPublicInputs,
  rollupProofLeft: Proof,
  rollupOutputRight: BaseOrMergeRollupPublicInputs,
  rollupProofRight: Proof,
  l1ToL2Roots: RootParityInput,
  newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
  db: MerkleTreeOperations,
) {
  const vks = getVerificationKeys();
  const vk = rollupOutputLeft.rollupType === RollupTypes.Base ? vks.baseRollupCircuit : vks.mergeRollupCircuit;
  const previousRollupData: RootRollupInputs['previousRollupData'] = [
    getPreviousRollupDataFromPublicInputs(rollupOutputLeft, rollupProofLeft, vk),
    getPreviousRollupDataFromPublicInputs(rollupOutputRight, rollupProofRight, vk),
  ];

  const getRootTreeSiblingPath = async (treeId: MerkleTreeId) => {
    const { size } = await db.getTreeInfo(treeId);
    const path = await db.getSiblingPath(treeId, size);
    return path.toFields();
  };

  const newL1ToL2MessageTreeRootSiblingPathArray = await getSubtreeSiblingPath(
    MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
    L1_TO_L2_MSG_SUBTREE_HEIGHT,
    db,
  );

  const newL1ToL2MessageTreeRootSiblingPath = makeTuple(
    L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
    i => (i < newL1ToL2MessageTreeRootSiblingPathArray.length ? newL1ToL2MessageTreeRootSiblingPathArray[i] : Fr.ZERO),
    0,
  );

  // Get tree snapshots
  const startL1ToL2MessageTreeSnapshot = await getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, db);

  // Get blocks tree
  const startArchiveSnapshot = await getTreeSnapshot(MerkleTreeId.ARCHIVE, db);
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

export function getPreviousRollupDataFromPublicInputs(
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

export async function getConstantRollupData(
  globalVariables: GlobalVariables,
  db: MerkleTreeOperations,
): Promise<ConstantRollupData> {
  return ConstantRollupData.from({
    baseRollupVkHash: DELETE_FR,
    mergeRollupVkHash: DELETE_FR,
    privateKernelVkTreeRoot: FUTURE_FR,
    publicKernelVkTreeRoot: FUTURE_FR,
    lastArchive: await getTreeSnapshot(MerkleTreeId.ARCHIVE, db),
    globalVariables,
  });
}

export async function getTreeSnapshot(id: MerkleTreeId, db: MerkleTreeOperations): Promise<AppendOnlyTreeSnapshot> {
  const treeInfo = await db.getTreeInfo(id);
  return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
}

export function getKernelDataFor(tx: ProcessedTx, vks: VerificationKeys): KernelData {
  return new KernelData(
    tx.data,
    tx.proof,

    // VK for the kernel circuit
    vks.privateKernelCircuit,

    // MembershipWitness for a VK tree to be implemented in the future
    FUTURE_NUM,
    assertLength(Array(VK_TREE_HEIGHT).fill(FUTURE_FR), VK_TREE_HEIGHT),
  );
}

export function makeEmptyMembershipWitness<N extends number>(height: N) {
  return new MembershipWitness(
    height,
    0n,
    makeTuple(height, () => Fr.ZERO),
  );
}

export async function processPublicDataUpdateRequests(tx: ProcessedTx, db: MerkleTreeOperations) {
  const combinedPublicDataUpdateRequests = tx.data.end.publicDataUpdateRequests.map(updateRequest => {
    return new PublicDataTreeLeaf(updateRequest.leafSlot, updateRequest.newValue);
  });
  const { lowLeavesWitnessData, newSubtreeSiblingPath, sortedNewLeaves, sortedNewLeavesIndexes } = await db.batchInsert(
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

  const lowPublicDataWritesPreimages: Tuple<PublicDataTreeLeafPreimage, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX> =
    makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
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

export async function getSubtreeSiblingPath(
  treeId: MerkleTreeId,
  subtreeHeight: number,
  db: MerkleTreeOperations,
): Promise<Fr[]> {
  const nextAvailableLeafIndex = await db.getTreeInfo(treeId).then(t => t.size);
  const fullSiblingPath = await db.getSiblingPath(treeId, nextAvailableLeafIndex);

  // Drop the first subtreeHeight items since we only care about the path to the subtree root
  return fullSiblingPath.getSubtreeSiblingPath(subtreeHeight).toFields();
}

// Scan a tree searching for a specific value and return a membership witness proof for it
export async function getMembershipWitnessFor<N extends number>(
  value: Fr,
  treeId: MerkleTreeId,
  height: N,
  db: MerkleTreeOperations,
): Promise<MembershipWitness<N>> {
  // If this is an empty tx, then just return zeroes
  if (value.isZero()) {
    return makeEmptyMembershipWitness(height);
  }

  const index = await db.findLeafIndex(treeId, value.toBuffer());
  if (index === undefined) {
    throw new Error(`Leaf with value ${value} not found in tree ${MerkleTreeId[treeId]}`);
  }
  const path = await db.getSiblingPath(treeId, index);
  return new MembershipWitness(height, index, assertLength(path.toFields(), height));
}

export async function executeBaseRollupCircuit(
  tx: ProcessedTx,
  inputs: BaseRollupInputs,
  treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>,
  simulator: RollupSimulator,
  prover: RollupProver,
  logger?: DebugLogger,
): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
  logger?.(`Running base rollup for ${tx.hash}`);
  const rollupOutput = await simulator.baseRollupCircuit(inputs);
  validatePartialState(rollupOutput.end, treeSnapshots);
  const proof = await prover.getBaseRollupProof(inputs, rollupOutput);
  return [rollupOutput, proof];
}

export function validatePartialState(
  partialState: PartialStateReference,
  treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>,
) {
  validateSimulatedTree(treeSnapshots.get(MerkleTreeId.NOTE_HASH_TREE)!, partialState.noteHashTree, 'NoteHashTree');
  validateSimulatedTree(treeSnapshots.get(MerkleTreeId.NULLIFIER_TREE)!, partialState.nullifierTree, 'NullifierTree');
  validateSimulatedTree(
    treeSnapshots.get(MerkleTreeId.PUBLIC_DATA_TREE)!,
    partialState.publicDataTree,
    'PublicDataTree',
  );
}

// Helper for comparing two trees snapshots
export function validateSimulatedTree(
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
      `${label ?? name} tree next available leaf index mismatch (local ${localTree.nextAvailableLeafIndex}, simulated ${
        simulatedTree.nextAvailableLeafIndex
      })`,
    );
  }
}

export async function executeBaseParityCircuit(
  inputs: BaseParityInputs,
  simulator: RollupSimulator,
  prover: RollupProver,
  logger?: DebugLogger,
): Promise<RootParityInput> {
  logger?.debug(`Running base parity circuit`);
  const parityPublicInputs = await simulator.baseParityCircuit(inputs);
  const proof = await prover.getBaseParityProof(inputs, parityPublicInputs);
  return new RootParityInput(proof, parityPublicInputs);
}

export async function executeRootParityCircuit(
  inputs: RootParityInputs,
  simulator: RollupSimulator,
  prover: RollupProver,
  logger?: DebugLogger,
): Promise<RootParityInput> {
  logger?.debug(`Running root parity circuit`);
  const parityPublicInputs = await simulator.rootParityCircuit(inputs);
  const proof = await prover.getRootParityProof(inputs, parityPublicInputs);
  return new RootParityInput(proof, parityPublicInputs);
}

export function validateTx(tx: ProcessedTx) {
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
