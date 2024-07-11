import { MerkleTreeId, type ProcessedTx } from '@aztec/circuit-types';
import {
  ARCHIVE_HEIGHT,
  AppendOnlyTreeSnapshot,
  type BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  ConstantRollupData,
  Fr,
  type GlobalVariables,
  KernelData,
  type L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  MAX_NULLIFIERS_PER_TX,
  MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MembershipWitness,
  MergeRollupInputs,
  type NESTED_RECURSIVE_PROOF_LENGTH,
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
  PublicDataHint,
  PublicDataTreeLeaf,
  type PublicDataTreeLeafPreimage,
  PublicDataUpdateRequest,
  type RECURSIVE_PROOF_LENGTH,
  type RecursiveProof,
  type RootParityInput,
  RootRollupInputs,
  type RootRollupPublicInputs,
  StateDiffHints,
  type StateReference,
  type TUBE_PROOF_LENGTH,
  VK_TREE_HEIGHT,
  type VerificationKeyAsFields,
  type VerificationKeyData,
} from '@aztec/circuits.js';
import { assertPermutation, makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { type Tuple, assertLength, toFriendlyJSON } from '@aztec/foundation/serialize';
import { getVKIndex, getVKSiblingPath, getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';
import { HintsBuilder, computeFeePayerBalanceLeafSlot } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

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
  proof: RecursiveProof<typeof TUBE_PROOF_LENGTH>,
  globalVariables: GlobalVariables,
  db: MerkleTreeOperations,
  kernelVk: VerificationKeyData,
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

  // Create data hint for reading fee payer initial balance in gas tokens
  // If no fee payer is set, read hint should be empty
  // If there is already a public data write for this slot, also skip the read hint
  const hintsBuilder = new HintsBuilder(db);
  const leafSlot = computeFeePayerBalanceLeafSlot(tx.data.feePayer);
  const existingBalanceWrite = tx.data.end.publicDataUpdateRequests.find(write => write.leafSlot.equals(leafSlot));
  const feePayerGasTokenBalanceReadHint =
    leafSlot.isZero() || existingBalanceWrite
      ? PublicDataHint.empty()
      : await hintsBuilder.getPublicDataHint(leafSlot.toBigInt());

  // Update the note hash trees with the new items being inserted to get the new roots
  // that will be used by the next iteration of the base rollup circuit, skipping the empty ones
  const noteHashes = tx.data.end.noteHashes;
  await db.appendLeaves(MerkleTreeId.NOTE_HASH_TREE, noteHashes);

  // The read witnesses for a given TX should be generated before the writes of the same TX are applied.
  // All reads that refer to writes in the same tx are transient and can be simplified out.
  const txPublicDataUpdateRequestInfo = await processPublicDataUpdateRequests(tx, db);

  // Update the nullifier tree, capturing the low nullifier info for each individual operation
  const {
    lowLeavesWitnessData: nullifierWitnessLeaves,
    newSubtreeSiblingPath: nullifiersSubtreeSiblingPath,
    sortedNewLeaves: sortednullifiers,
    sortedNewLeavesIndexes,
  } = await db.batchInsert(
    MerkleTreeId.NULLIFIER_TREE,
    tx.data.end.nullifiers.map(n => n.toBuffer()),
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

  const nullifierSubtreeSiblingPathArray = nullifiersSubtreeSiblingPath.toFields();

  const nullifierSubtreeSiblingPath = makeTuple(NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, i =>
    i < nullifierSubtreeSiblingPathArray.length ? nullifierSubtreeSiblingPathArray[i] : Fr.ZERO,
  );

  const publicDataSiblingPath = txPublicDataUpdateRequestInfo.newPublicDataSubtreeSiblingPath;

  const stateDiffHints = StateDiffHints.from({
    nullifierPredecessorPreimages: makeTuple(MAX_NULLIFIERS_PER_TX, i =>
      i < nullifierWitnessLeaves.length
        ? (nullifierWitnessLeaves[i].leafPreimage as NullifierLeafPreimage)
        : NullifierLeafPreimage.empty(),
    ),
    nullifierPredecessorMembershipWitnesses: makeTuple(MAX_NULLIFIERS_PER_TX, i =>
      i < nullifierPredecessorMembershipWitnessesWithoutPadding.length
        ? nullifierPredecessorMembershipWitnessesWithoutPadding[i]
        : makeEmptyMembershipWitness(NULLIFIER_TREE_HEIGHT),
    ),
    sortedNullifiers: makeTuple(MAX_NULLIFIERS_PER_TX, i => Fr.fromBuffer(sortednullifiers[i])),
    sortedNullifierIndexes: makeTuple(MAX_NULLIFIERS_PER_TX, i => sortedNewLeavesIndexes[i]),
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
    kernelData: getKernelDataFor(tx, kernelVk, proof),
    start,
    stateDiffHints,
    feePayerGasTokenBalanceReadHint,
    sortedPublicDataWrites: txPublicDataUpdateRequestInfo.sortedPublicDataWrites,
    sortedPublicDataWritesIndexes: txPublicDataUpdateRequestInfo.sortedPublicDataWritesIndexes,
    lowPublicDataWritesPreimages: txPublicDataUpdateRequestInfo.lowPublicDataWritesPreimages,
    lowPublicDataWritesMembershipWitnesses: txPublicDataUpdateRequestInfo.lowPublicDataWritesMembershipWitnesses,

    archiveRootMembershipWitness,

    constants,
  });
}

export function createMergeRollupInputs(
  left: [BaseOrMergeRollupPublicInputs, RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>, VerificationKeyAsFields],
  right: [BaseOrMergeRollupPublicInputs, RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>, VerificationKeyAsFields],
) {
  const mergeInputs = new MergeRollupInputs([
    getPreviousRollupDataFromPublicInputs(left[0], left[1], left[2]),
    getPreviousRollupDataFromPublicInputs(right[0], right[1], right[2]),
  ]);
  return mergeInputs;
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
  rollupProofLeft: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
  verificationKeyLeft: VerificationKeyAsFields,
  rollupOutputRight: BaseOrMergeRollupPublicInputs,
  rollupProofRight: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
  verificationKeyRight: VerificationKeyAsFields,
  l1ToL2Roots: RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
  newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
  messageTreeSnapshot: AppendOnlyTreeSnapshot,
  messageTreeRootSiblingPath: Tuple<Fr, typeof L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH>,
  db: MerkleTreeOperations,
) {
  const previousRollupData: RootRollupInputs['previousRollupData'] = [
    getPreviousRollupDataFromPublicInputs(rollupOutputLeft, rollupProofLeft, verificationKeyLeft),
    getPreviousRollupDataFromPublicInputs(rollupOutputRight, rollupProofRight, verificationKeyRight),
  ];

  const getRootTreeSiblingPath = async (treeId: MerkleTreeId) => {
    const { size } = await db.getTreeInfo(treeId);
    const path = await db.getSiblingPath(treeId, size);
    return path.toFields();
  };

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
    newL1ToL2MessageTreeRootSiblingPath: messageTreeRootSiblingPath,
    startL1ToL2MessageTreeSnapshot: messageTreeSnapshot,
    startArchiveSnapshot,
    newArchiveSiblingPath,
  });
}

export function getPreviousRollupDataFromPublicInputs(
  rollupOutput: BaseOrMergeRollupPublicInputs,
  rollupProof: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
  vk: VerificationKeyAsFields,
) {
  const leafIndex = getVKIndex(vk);

  return new PreviousRollupData(
    rollupOutput,
    rollupProof,
    vk,
    new MembershipWitness(VK_TREE_HEIGHT, BigInt(leafIndex), getVKSiblingPath(leafIndex)),
  );
}

export async function getConstantRollupData(
  globalVariables: GlobalVariables,
  db: MerkleTreeOperations,
): Promise<ConstantRollupData> {
  return ConstantRollupData.from({
    vkTreeRoot: getVKTreeRoot(),
    lastArchive: await getTreeSnapshot(MerkleTreeId.ARCHIVE, db),
    globalVariables,
  });
}

export async function getTreeSnapshot(id: MerkleTreeId, db: MerkleTreeOperations): Promise<AppendOnlyTreeSnapshot> {
  const treeInfo = await db.getTreeInfo(id);
  return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
}

export function getKernelDataFor(
  tx: ProcessedTx,
  vk: VerificationKeyData,
  proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>,
): KernelData {
  const leafIndex = getVKIndex(vk);

  return new KernelData(
    tx.data,
    proof,
    // VK for the kernel circuit
    vk,
    leafIndex,
    getVKSiblingPath(leafIndex),
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
  const allPublicDataUpdateRequests = padArrayEnd(
    tx.finalPublicDataUpdateRequests,
    PublicDataUpdateRequest.empty(),
    MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  );

  const allPublicDataWrites = allPublicDataUpdateRequests.map(
    ({ leafSlot, newValue }) => new PublicDataTreeLeaf(leafSlot, newValue),
  );
  const { lowLeavesWitnessData, newSubtreeSiblingPath, sortedNewLeaves, sortedNewLeavesIndexes } = await db.batchInsert(
    MerkleTreeId.PUBLIC_DATA_TREE,
    allPublicDataWrites.map(x => x.toBuffer()),
    // TODO(#3675) remove oldValue from update requests
    PUBLIC_DATA_SUBTREE_HEIGHT,
  );

  if (lowLeavesWitnessData === undefined) {
    throw new Error(`Could not craft public data batch insertion proofs`);
  }

  const sortedPublicDataWrites = makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
    return PublicDataTreeLeaf.fromBuffer(sortedNewLeaves[i]);
  });

  const sortedPublicDataWritesIndexes = makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
    return sortedNewLeavesIndexes[i];
  });

  const subtreeSiblingPathAsFields = newSubtreeSiblingPath.toFields();
  const newPublicDataSubtreeSiblingPath = makeTuple(PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, i => {
    return subtreeSiblingPathAsFields[i];
  });

  const lowPublicDataWritesMembershipWitnesses: Tuple<
    MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
    typeof MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
  > = makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
    const witness = lowLeavesWitnessData[i];
    return MembershipWitness.fromBufferArray(
      witness.index,
      assertLength(witness.siblingPath.toBufferArray(), PUBLIC_DATA_TREE_HEIGHT),
    );
  });

  const lowPublicDataWritesPreimages: Tuple<
    PublicDataTreeLeafPreimage,
    typeof MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
  > = makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => {
    return lowLeavesWitnessData[i].leafPreimage as PublicDataTreeLeafPreimage;
  });

  // validate that the sortedPublicDataWrites and sortedPublicDataWritesIndexes are in the correct order
  // otherwise it will just fail in the circuit
  assertPermutation(allPublicDataWrites, sortedPublicDataWrites, sortedPublicDataWritesIndexes, (a, b) => a.equals(b));

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
