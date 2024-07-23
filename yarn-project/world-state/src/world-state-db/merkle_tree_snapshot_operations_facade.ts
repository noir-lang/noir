import { MerkleTreeId, type SiblingPath } from '@aztec/circuit-types';
import { AppendOnlyTreeSnapshot, Fr, type Header, PartialStateReference, StateReference } from '@aztec/circuits.js';
import { type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';
import { type BatchInsertionResult, type IndexedTreeSnapshot } from '@aztec/merkle-tree';

import { type MerkleTreeDb, type TreeSnapshots } from './merkle_tree_db.js';
import {
  type HandleL2BlockAndMessagesResult,
  type IndexedTreeId,
  type MerkleTreeLeafType,
  type MerkleTreeOperations,
  type TreeInfo,
} from './merkle_tree_operations.js';

/**
 * Merkle tree operations on readonly tree snapshots.
 */
export class MerkleTreeSnapshotOperationsFacade implements MerkleTreeOperations {
  #treesDb: MerkleTreeDb;
  #blockNumber: number;
  #treeSnapshots: TreeSnapshots = {} as any;

  constructor(trees: MerkleTreeDb, blockNumber: number) {
    this.#treesDb = trees;
    this.#blockNumber = blockNumber;
  }

  async #getTreeSnapshot(treeId: MerkleTreeId): Promise<TreeSnapshots[typeof treeId]> {
    if (this.#treeSnapshots[treeId]) {
      return this.#treeSnapshots[treeId];
    }

    this.#treeSnapshots = await this.#treesDb.getSnapshot(this.#blockNumber);
    return this.#treeSnapshots[treeId]!;
  }

  async findLeafIndex<ID extends MerkleTreeId>(treeId: ID, value: MerkleTreeLeafType<ID>): Promise<bigint | undefined> {
    const tree = await this.#getTreeSnapshot(treeId);
    // TODO #5448 fix "as any"
    return tree.findLeafIndex(value as any);
  }

  async findLeafIndexAfter<ID extends MerkleTreeId>(
    treeId: MerkleTreeId,
    value: MerkleTreeLeafType<ID>,
    startIndex: bigint,
  ): Promise<bigint | undefined> {
    const tree = await this.#getTreeSnapshot(treeId);
    // TODO #5448 fix "as any"
    return tree.findLeafIndexAfter(value as any, startIndex);
  }

  async getLeafPreimage<ID extends IndexedTreeId>(
    treeId: ID,
    index: bigint,
  ): Promise<IndexedTreeLeafPreimage | undefined> {
    const snapshot = (await this.#getTreeSnapshot(treeId)) as IndexedTreeSnapshot;
    return snapshot.getLatestLeafPreimageCopy(BigInt(index));
  }

  async getLeafValue<ID extends MerkleTreeId>(
    treeId: ID,
    index: bigint,
  ): Promise<MerkleTreeLeafType<typeof treeId> | undefined> {
    const snapshot = await this.#getTreeSnapshot(treeId);
    return snapshot.getLeafValue(BigInt(index)) as MerkleTreeLeafType<typeof treeId> | undefined;
  }

  async getPreviousValueIndex(
    treeId: IndexedTreeId,
    value: bigint,
  ): Promise<
    | {
        /**
         * The index of the found leaf.
         */
        index: bigint;
        /**
         * A flag indicating if the corresponding leaf's value is equal to `newValue`.
         */
        alreadyPresent: boolean;
      }
    | undefined
  > {
    const snapshot = (await this.#getTreeSnapshot(treeId)) as IndexedTreeSnapshot;
    return snapshot.findIndexOfPreviousKey(value);
  }

  async getSiblingPath<N extends number>(treeId: MerkleTreeId, index: bigint): Promise<SiblingPath<N>> {
    const snapshot = await this.#getTreeSnapshot(treeId);
    return snapshot.getSiblingPath(index);
  }

  async getTreeInfo(treeId: MerkleTreeId): Promise<TreeInfo> {
    const snapshot = await this.#getTreeSnapshot(treeId);
    return {
      depth: snapshot.getDepth(),
      root: snapshot.getRoot(),
      size: snapshot.getNumLeaves(),
      treeId,
    };
  }

  async getStateReference(): Promise<StateReference> {
    const snapshots = await Promise.all([
      this.#getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE),
      this.#getTreeSnapshot(MerkleTreeId.NOTE_HASH_TREE),
      this.#getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE),
      this.#getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE),
      this.#getTreeSnapshot(MerkleTreeId.ARCHIVE),
    ]);

    return new StateReference(
      new AppendOnlyTreeSnapshot(
        Fr.fromBuffer(snapshots[MerkleTreeId.L1_TO_L2_MESSAGE_TREE].getRoot()),
        Number(snapshots[MerkleTreeId.L1_TO_L2_MESSAGE_TREE].getNumLeaves()),
      ),
      new PartialStateReference(
        new AppendOnlyTreeSnapshot(
          Fr.fromBuffer(snapshots[MerkleTreeId.NOTE_HASH_TREE].getRoot()),
          Number(snapshots[MerkleTreeId.NOTE_HASH_TREE].getNumLeaves()),
        ),
        new AppendOnlyTreeSnapshot(
          Fr.fromBuffer(snapshots[MerkleTreeId.NULLIFIER_TREE].getRoot()),
          Number(snapshots[MerkleTreeId.NULLIFIER_TREE].getNumLeaves()),
        ),
        new AppendOnlyTreeSnapshot(
          Fr.fromBuffer(snapshots[MerkleTreeId.PUBLIC_DATA_TREE].getRoot()),
          Number(snapshots[MerkleTreeId.PUBLIC_DATA_TREE].getNumLeaves()),
        ),
      ),
    );
  }

  appendLeaves(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  batchInsert<TreeHeight extends number, SubtreeSiblingPathHeight extends number>(): Promise<
    BatchInsertionResult<TreeHeight, SubtreeSiblingPathHeight>
  > {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  updateArchive(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  commit(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  handleL2BlockAndMessages(): Promise<HandleL2BlockAndMessagesResult> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  rollback(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  updateHistoricArchive(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  updateLeaf(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  getInitialHeader(): Header {
    throw new Error('Getting initial header not supported on snapshot.');
  }
}
