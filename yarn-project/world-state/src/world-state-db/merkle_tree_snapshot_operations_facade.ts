import { MerkleTreeId } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { IndexedTreeLeafPreimage } from '@aztec/foundation/trees';
import { BatchInsertionResult, IndexedTreeSnapshot, TreeSnapshot } from '@aztec/merkle-tree';
import { SiblingPath } from '@aztec/types/membership';

import { MerkleTreeDb } from './merkle_tree_db.js';
import { CurrentTreeRoots, HandleL2BlockResult, MerkleTreeOperations, TreeInfo } from './merkle_tree_operations.js';

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

  getLatestGlobalVariablesHash(): Promise<Fr> {
    return Promise.reject(new Error('not implemented'));
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

  async getTreeRoots(): Promise<CurrentTreeRoots> {
    const snapshots = await Promise.all([
      this.#getTreeSnapshot(MerkleTreeId.CONTRACT_TREE),
      this.#getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE),
      this.#getTreeSnapshot(MerkleTreeId.NOTE_HASH_TREE),
      this.#getTreeSnapshot(MerkleTreeId.PUBLIC_DATA_TREE),
      this.#getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE),
      this.#getTreeSnapshot(MerkleTreeId.ARCHIVE),
    ]);

    return {
      archiveRoot: snapshots[MerkleTreeId.ARCHIVE].getRoot(),
      contractDataTreeRoot: snapshots[MerkleTreeId.CONTRACT_TREE].getRoot(),
      l1Tol2MessageTreeRoot: snapshots[MerkleTreeId.L1_TO_L2_MESSAGE_TREE].getRoot(),
      noteHashTreeRoot: snapshots[MerkleTreeId.NOTE_HASH_TREE].getRoot(),
      nullifierTreeRoot: snapshots[MerkleTreeId.NULLIFIER_TREE].getRoot(),
      publicDataTreeRoot: snapshots[MerkleTreeId.PUBLIC_DATA_TREE].getRoot(),
    };
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

  handleL2Block(): Promise<HandleL2BlockResult> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  rollback(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  updateHistoricArchive(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  updateLatestGlobalVariablesHash(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }

  updateLeaf(): Promise<void> {
    return Promise.reject(new Error('Tree snapshot operations are read-only'));
  }
}
