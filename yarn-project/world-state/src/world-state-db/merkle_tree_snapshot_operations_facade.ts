import { MerkleTreeId, SiblingPath } from '@aztec/circuit-types';
import { AppendOnlyTreeSnapshot, Fr, Header, PartialStateReference, StateReference } from '@aztec/circuits.js';
import { IndexedTreeLeafPreimage } from '@aztec/foundation/trees';
import { BatchInsertionResult, IndexedTreeSnapshot, TreeSnapshot } from '@aztec/merkle-tree';

import { MerkleTreeDb } from './merkle_tree_db.js';
import { HandleL2BlockAndMessagesResult, MerkleTreeOperations, TreeInfo } from './merkle_tree_operations.js';

/**
 * Merkle tree operations on readonly tree snapshots.
 */
export class MerkleTreeSnapshotOperationsFacade implements MerkleTreeOperations {
  #treesDb: MerkleTreeDb;
  #blockNumber: number;
  #treeSnapshots: ReadonlyArray<TreeSnapshot | IndexedTreeSnapshot> = [];

  constructor(trees: MerkleTreeDb, blockNumber: number) {
    this.#treesDb = trees;
    this.#blockNumber = blockNumber;
  }

  async #getTreeSnapshot(merkleTreeId: number): Promise<TreeSnapshot | IndexedTreeSnapshot> {
    if (this.#treeSnapshots[merkleTreeId]) {
      return this.#treeSnapshots[merkleTreeId];
    }

    this.#treeSnapshots = await this.#treesDb.getSnapshot(this.#blockNumber);
    return this.#treeSnapshots[merkleTreeId]!;
  }

  async findLeafIndex(treeId: MerkleTreeId, value: Buffer): Promise<bigint | undefined> {
    const tree = await this.#getTreeSnapshot(treeId);
    return tree.findLeafIndex(value);
  }

  async getLeafPreimage(
    treeId: MerkleTreeId.NULLIFIER_TREE,
    index: bigint,
  ): Promise<IndexedTreeLeafPreimage | undefined> {
    const snapshot = (await this.#getTreeSnapshot(treeId)) as IndexedTreeSnapshot;
    return snapshot.getLatestLeafPreimageCopy(BigInt(index));
  }

  async getLeafValue(treeId: MerkleTreeId, index: bigint): Promise<Buffer | undefined> {
    const snapshot = await this.#getTreeSnapshot(treeId);
    return snapshot.getLeafValue(BigInt(index));
  }

  async getPreviousValueIndex(
    treeId: MerkleTreeId.NULLIFIER_TREE,
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

  buildInitialHeader(): Promise<Header> {
    throw new Error('Building initial header not supported on snapshot.');
  }
}
