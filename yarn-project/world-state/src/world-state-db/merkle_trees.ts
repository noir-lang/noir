import { PrimitivesWasm } from '@aztec/barretenberg.js/wasm';
import {
  CONTRACT_TREE_HEIGHT,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT,
  L1_TO_L2_MESSAGES_TREE_HEIGHT,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_DATA_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { SerialQueue } from '@aztec/foundation';
import { WasmWrapper } from '@aztec/foundation/wasm';
import {
  AppendOnlyTree,
  StandardIndexedTree,
  LeafData,
  Pedersen,
  SiblingPath,
  StandardTree,
  UpdateOnlyTree,
  IndexedTree,
  newTree,
  SparseTree,
} from '@aztec/merkle-tree';
import { default as levelup } from 'levelup';
import { MerkleTreeOperationsFacade } from '../merkle-tree/merkle_tree_operations_facade.js';
import {
  INITIAL_NULLIFIER_TREE_SIZE,
  IndexedTreeId,
  MerkleTreeDb,
  MerkleTreeId,
  MerkleTreeOperations,
  PublicTreeId,
  TreeInfo,
} from './index.js';

/**
 * A convenience class for managing multiple merkle trees.
 */
export class MerkleTrees implements MerkleTreeDb {
  private trees: (AppendOnlyTree | UpdateOnlyTree)[] = [];
  private jobQueue = new SerialQueue();

  constructor(private db: levelup.LevelUp) {}

  /**
   * Initialises the collection of Merkle Trees.
   * @param optionalWasm - WASM instance to use for hashing (if not provided PrimitivesWasm will be used).
   */
  public async init(optionalWasm?: WasmWrapper) {
    const wasm = optionalWasm ?? (await PrimitivesWasm.get());
    const hasher = new Pedersen(wasm);
    const contractTree: AppendOnlyTree = await newTree(
      StandardTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.CONTRACT_TREE]}`,
      CONTRACT_TREE_HEIGHT,
    );
    const contractTreeRootsTree: AppendOnlyTree = await newTree(
      StandardTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.CONTRACT_TREE_ROOTS_TREE]}`,
      CONTRACT_TREE_ROOTS_TREE_HEIGHT,
    );
    const nullifierTree = await newTree(
      StandardIndexedTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.NULLIFIER_TREE]}`,
      NULLIFIER_TREE_HEIGHT,
      INITIAL_NULLIFIER_TREE_SIZE,
    );
    const privateDataTree: AppendOnlyTree = await newTree(
      StandardTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.PRIVATE_DATA_TREE]}`,
      PRIVATE_DATA_TREE_HEIGHT,
    );
    const privateDataTreeRootsTree: AppendOnlyTree = await newTree(
      StandardTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE]}`,
      PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
    );
    const publicDataTree: UpdateOnlyTree = await newTree(
      SparseTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.PUBLIC_DATA_TREE]}`,
      PUBLIC_DATA_TREE_HEIGHT,
    );
    const l1Tol2MessagesTree: AppendOnlyTree = await newTree(
      StandardTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.L1_TO_L2_MESSAGES_TREE]}`,
      L1_TO_L2_MESSAGES_TREE_HEIGHT,
    );
    const l1Tol2MessagesRootsTree: AppendOnlyTree = await newTree(
      StandardTree,
      this.db,
      hasher,
      `${MerkleTreeId[MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE]}`,
      L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT,
    );
    this.trees = [
      contractTree,
      contractTreeRootsTree,
      nullifierTree,
      privateDataTree,
      privateDataTreeRootsTree,
      publicDataTree,
      l1Tol2MessagesTree,
      l1Tol2MessagesRootsTree,
    ];

    this.jobQueue.start();

    await this.updateHistoricRootsTrees(true);
  }

  /**
   * Method to asynchronously create and initialise a MerkleTrees instance.
   * @param db - The db instance to use for data persistance.
   * @param wasm - WASM instance to use for hashing (if not provided PrimitivesWasm will be used).
   * @returns - A fully initialised MerkleTrees instance.
   */
  public static async new(db: levelup.LevelUp, wasm?: WasmWrapper) {
    const merkleTrees = new MerkleTrees(db);
    await merkleTrees.init(wasm);
    return merkleTrees;
  }

  /**
   * Stops the job queue (waits for all jobs to finish).
   */
  public async stop() {
    await this.jobQueue.end();
  }

  /**
   * Gets a view of this db that returns uncommitted data.
   * @returns - A facade for this instance.
   */
  public asLatest(): MerkleTreeOperations {
    return new MerkleTreeOperationsFacade(this, true);
  }

  /**
   * Gets a view of this db that returns committed data only.
   * @returns - A facade for this instance.
   */
  public asCommitted(): MerkleTreeOperations {
    return new MerkleTreeOperationsFacade(this, false);
  }

  /**
   * Inserts into the roots trees (CONTRACT_TREE_ROOTS_TREE, PRIVATE_DATA_TREE_ROOTS_TREE)
   * the current roots of the corresponding trees (CONTRACT_TREE, PRIVATE_DATA_TREE).
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   */
  public async updateHistoricRootsTrees(includeUncommitted: boolean) {
    for (const [newTree, rootTree] of [
      [MerkleTreeId.PRIVATE_DATA_TREE, MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE],
      [MerkleTreeId.CONTRACT_TREE, MerkleTreeId.CONTRACT_TREE_ROOTS_TREE],
      [MerkleTreeId.L1_TO_L2_MESSAGES_TREE, MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE],
    ] as const) {
      const newTreeInfo = await this.getTreeInfo(newTree, includeUncommitted);
      await this.appendLeaves(rootTree, [newTreeInfo.root]);
    }
  }

  /**
   * Gets the tree info for the specified tree.
   * @param treeId - Id of the tree to get information from.
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   * @returns The tree info for the specified tree.
   */
  public async getTreeInfo(treeId: MerkleTreeId, includeUncommitted: boolean): Promise<TreeInfo> {
    return await this.synchronise(() => this._getTreeInfo(treeId, includeUncommitted));
  }

  /**
   * Gets the value at the given index.
   * @param treeId - The ID of the tree to get the leaf value from.
   * @param index - The index of the leaf.
   * @param includeUncommitted - Indicates whether to include uncommitted changes.
   * @returns Leaf value at the given index (undefined if not found).
   */
  public async getLeafValue(
    treeId: MerkleTreeId,
    index: bigint,
    includeUncommitted: boolean,
  ): Promise<Buffer | undefined> {
    return await this.synchronise(() => this.trees[treeId].getLeafValue(index, includeUncommitted));
  }

  /**
   * Gets the sibling path for a leaf in a tree.
   * @param treeId - The ID of the tree.
   * @param index - The index of the leaf.
   * @param includeUncommitted - Indicates whether the sibling path should incro include uncommitted data.
   * @returns The sibling path for the leaf.
   */
  public async getSiblingPath(treeId: MerkleTreeId, index: bigint, includeUncommitted: boolean): Promise<SiblingPath> {
    return await this.synchronise(() => this._getSiblingPath(treeId, index, includeUncommitted));
  }

  /**
   * Appends leaves to a tree.
   * @param treeId - The ID of the tree.
   * @param leaves - The leaves to append.
   * @returns Empty promise.
   */
  public async appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void> {
    return await this.synchronise(() => this._appendLeaves(treeId, leaves));
  }

  /**
   * Commits all pending updates.
   * @returns Empty promise.
   */
  public async commit(): Promise<void> {
    return await this.synchronise(() => this._commit());
  }

  /**
   * Rolls back all pending updates.
   * @returns Empty promise.
   */
  public async rollback(): Promise<void> {
    return await this.synchronise(() => this._rollback());
  }

  /**
   * Finds the index of the largest leaf whose value is less than or equal to the provided value.
   * @param treeId - The ID of the tree to search.
   * @param value - The value to be inserted into the tree.
   * @param includeUncommitted - If true, the uncommitted changes are included in the search.
   * @returns The found leaf index and a flag indicating if the corresponding leaf's value is equal to `newValue`.
   */
  public async getPreviousValueIndex(
    treeId: IndexedTreeId,
    value: bigint,
    includeUncommitted: boolean,
  ): Promise<{
    /**
     * The index of the found leaf.
     */
    index: number;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  }> {
    return await this.synchronise(() =>
      Promise.resolve(this._getIndexedTree(treeId).findIndexOfPreviousValue(value, includeUncommitted)),
    );
  }

  /**
   * Gets the leaf data at a given index and tree.
   * @param treeId - The ID of the tree get the leaf from.
   * @param index - The index of the leaf to get.
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   * @returns Leaf data.
   */
  public async getLeafData(
    treeId: IndexedTreeId,
    index: number,
    includeUncommitted: boolean,
  ): Promise<LeafData | undefined> {
    return await this.synchronise(() =>
      Promise.resolve(this._getIndexedTree(treeId).getLatestLeafDataCopy(index, includeUncommitted)),
    );
  }

  /**
   * Returns the index of a leaf given its value, or undefined if no leaf with that value is found.
   * @param treeId - The ID of the tree.
   * @param value - The leaf value to look for.
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   * @returns The index of the first leaf found with a given value (undefined if not found).
   */
  public async findLeafIndex(
    treeId: MerkleTreeId,
    value: Buffer,
    includeUncommitted: boolean,
  ): Promise<bigint | undefined> {
    return await this.synchronise(async () => {
      const tree = this.trees[treeId];
      for (let i = 0n; i < tree.getNumLeaves(includeUncommitted); i++) {
        const currentValue = await tree.getLeafValue(i, includeUncommitted);
        if (currentValue && currentValue.equals(value)) {
          return i;
        }
      }
      return undefined;
    });
  }

  /**
   * Updates a leaf in a tree at a given index.
   * @param treeId - The ID of the tree.
   * @param leaf - The new leaf value.
   * @param index - The index to insert into.
   * @returns Empty promise.
   */
  public async updateLeaf(treeId: IndexedTreeId | PublicTreeId, leaf: LeafData | Buffer, index: bigint): Promise<void> {
    const tree = this.trees[treeId];
    if (!('updateLeaf' in tree)) {
      throw new Error('Tree does not support `updateLeaf` method');
    }
    return await this.synchronise(() => tree.updateLeaf(leaf, index));
  }

  /**
   * Waits for all jobs to finish before executing the given function.
   * @param fn - The function to execute.
   * @returns Promise containing the result of the function.
   */
  private async synchronise<T>(fn: () => Promise<T>): Promise<T> {
    return await this.jobQueue.put(fn);
  }

  /**
   * Returns the tree info for the specified tree id.
   * @param treeId - Id of the tree to get information from.
   * @param includeUncommitted - Indicates whether to include uncommitted data.
   * @returns The tree info for the specified tree.
   */
  private _getTreeInfo(treeId: MerkleTreeId, includeUncommitted: boolean): Promise<TreeInfo> {
    const treeInfo = {
      treeId,
      root: this.trees[treeId].getRoot(includeUncommitted),
      size: this.trees[treeId].getNumLeaves(includeUncommitted),
      depth: this.trees[treeId].getDepth(),
    } as TreeInfo;
    return Promise.resolve(treeInfo);
  }

  /**
   * Returns an instance of an indexed tree.
   * @param treeId - Id of the tree to get an instance of.
   * @returns The indexed tree for the specified tree id.
   */
  private _getIndexedTree(treeId: IndexedTreeId): IndexedTree {
    return this.trees[treeId] as IndexedTree;
  }

  /**
   * Returns the sibling path for a leaf in a tree.
   * @param treeId - Id of the tree to get the sibling path from.
   * @param index - Index of the leaf to get the sibling path for.
   * @param includeUncommitted - Indicates whether to include uncommitted updates in the sibling path.
   * @returns Promise containing the sibling path for the leaf.
   */
  private _getSiblingPath(treeId: MerkleTreeId, index: bigint, includeUncommitted: boolean): Promise<SiblingPath> {
    return Promise.resolve(this.trees[treeId].getSiblingPath(index, includeUncommitted));
  }

  /**
   * Appends leaves to a tree.
   * @param treeId - Id of the tree to append leaves to.
   * @param leaves - Leaves to append.
   * @returns Empty promise.
   */
  private async _appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void> {
    const tree = this.trees[treeId];
    if (!('appendLeaves' in tree)) {
      throw new Error('Tree does not support `appendLeaves` method');
    }
    return await tree.appendLeaves(leaves);
  }

  /**
   * Commits all pending updates.
   * @returns Empty promise.
   */
  private async _commit(): Promise<void> {
    for (const tree of this.trees) {
      await tree.commit();
    }
  }

  /**
   * Rolls back all pending updates.
   * @returns Empty promise.
   */
  private async _rollback(): Promise<void> {
    for (const tree of this.trees) {
      await tree.rollback();
    }
  }
}
