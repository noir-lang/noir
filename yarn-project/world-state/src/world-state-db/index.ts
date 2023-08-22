import { MAX_NEW_NULLIFIERS_PER_TX } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { LeafData, LowLeafWitnessData } from '@aztec/merkle-tree';
import { L2Block, MerkleTreeId, SiblingPath } from '@aztec/types';

export * from './merkle_trees.js';
export { LeafData } from '@aztec/merkle-tree';

/**
 * Type alias for the nullifier tree ID.
 */
export type IndexedTreeId = MerkleTreeId.NULLIFIER_TREE;

/**
 * Type alias for the public data tree ID.
 */
export type PublicTreeId = MerkleTreeId.PUBLIC_DATA_TREE;

/**
 *
 * @remarks Short explanation:
 *    The nullifier tree must be initially padded as the pre-populated 0 index prevents efficient subtree insertion.
 *    Padding with some values solves this issue.
 *
 * @remarks Thorough explanation:
 *    There needs to be an initial (0,0,0) leaf in the tree, so that when we insert the first 'proper' leaf, we can
 *    prove that any value greater than 0 doesn't exist in the tree yet. We prefill/pad the tree with "the number of
 *    leaves that are added by one block" so that the first 'proper' block can insert a full subtree.
 *
 *    Without this padding, there would be a leaf (0,0,0) at leaf index 0, making it really difficult to insert e.g.
 *    1024 leaves for the first block, because there's only neat space for 1023 leaves after 0. By padding with 1023
 *    more leaves, we can then insert the first block of 1024 leaves into indices 1024:2047.
 */
export const INITIAL_NULLIFIER_TREE_SIZE = 2 * MAX_NEW_NULLIFIERS_PER_TX;

/**
 *  Defines tree information.
 */
export interface TreeInfo {
  /**
   * The tree ID.
   */
  treeId: MerkleTreeId;
  /**
   * The tree root.
   */
  root: Buffer;
  /**
   * The number of leaves in the tree.
   */
  size: bigint;

  /**
   * The depth of the tree.
   */
  depth: number;
}

/**
 * Adds a last boolean flag in each function on the type.
 */
type WithIncludeUncommitted<F> = F extends (...args: [...infer Rest]) => infer Return
  ? (...args: [...Rest, boolean]) => Return
  : F;

/**
 * The current roots of the commitment trees
 */
export type CurrentTreeRoots = {
  /** Private data tree root. */
  privateDataTreeRoot: Buffer;
  /** Contract data tree root. */
  contractDataTreeRoot: Buffer;
  /** L1 to L2 Messages data tree root. */
  l1Tol2MessagesTreeRoot: Buffer;
  /** Nullifier data tree root. */
  nullifierTreeRoot: Buffer;
  /** Blocks tree root. */
  blocksTreeRoot: Buffer;
  /** Public data tree root */
  publicDataTreeRoot: Buffer;
};

/**
 * Defines the names of the setters on Merkle Trees.
 */
type MerkleTreeSetters = 'appendLeaves' | 'updateLeaf' | 'commit' | 'rollback' | 'handleL2Block' | 'batchInsert';

/**
 * Defines the interface for operations on a set of Merkle Trees configuring whether to return committed or uncommitted data.
 */
export type MerkleTreeDb = {
  [Property in keyof MerkleTreeOperations as Exclude<Property, MerkleTreeSetters>]: WithIncludeUncommitted<
    MerkleTreeOperations[Property]
  >;
} & Pick<MerkleTreeOperations, MerkleTreeSetters>;

/**
 * Defines the interface for operations on a set of Merkle Trees.
 */
export interface MerkleTreeOperations {
  /**
   * Appends leaves to a given tree.
   * @param treeId - The tree to be updated.
   * @param leaves - The set of leaves to be appended.
   */
  appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void>;

  /**
   * Returns information about the given tree.
   * @param treeId - The tree to be queried.
   */
  getTreeInfo(treeId: MerkleTreeId): Promise<TreeInfo>;

  /**
   * Gets the current roots of the commitment trees.
   */
  getTreeRoots(): Promise<CurrentTreeRoots>;

  /**
   * Gets sibling path for a leaf.
   * @param treeId - The tree to be queried for a sibling path.
   * @param index - The index of the leaf for which a sibling path should be returned.
   */
  getSiblingPath<N extends number>(treeId: MerkleTreeId, index: bigint): Promise<SiblingPath<N>>;

  /**
   * Returns the previous index for a given value in an indexed tree.
   * @param treeId - The tree for which the previous value index is required.
   * @param value - The value to be queried.
   */
  getPreviousValueIndex(
    treeId: IndexedTreeId,
    value: bigint,
  ): Promise<{
    /**
     * The index of the found leaf.
     */
    index: number;
    /**
     * A flag indicating if the corresponding leaf's value is equal to `newValue`.
     */
    alreadyPresent: boolean;
  }>;

  /**
   * Returns the data at a specific leaf.
   * @param treeId - The tree for which leaf data should be returned.
   * @param index - The index of the leaf required.
   */
  getLeafData(treeId: IndexedTreeId, index: number): Promise<LeafData | undefined>;

  /**
   * Update the leaf data at the given index.
   * @param treeId - The tree for which leaf data should be edited.
   * @param leaf - The updated leaf value.
   * @param index - The index of the leaf to be updated.
   */
  updateLeaf(treeId: IndexedTreeId | PublicTreeId, leaf: LeafData | Buffer, index: bigint): Promise<void>;

  /**
   * Returns the index containing a leaf value.
   * @param treeId - The tree for which the index should be returned.
   * @param value - The value to search for in the tree.
   */
  findLeafIndex(treeId: MerkleTreeId, value: Buffer): Promise<bigint | undefined>;

  /**
   * Gets the value for a leaf in the tree.
   * @param treeId - The tree for which the index should be returned.
   * @param index - The index of the leaf.
   */
  getLeafValue(treeId: MerkleTreeId, index: bigint): Promise<Buffer | undefined>;

  /**
   * Inserts the new block hash into the new block hashes tree.
   * This includes all of the current roots of all of the data trees and the current blocks global vars.
   * @param globalVariablesHash - The global variables hash to insert into the block hash.
   */
  updateHistoricBlocksTree(globalVariablesHash: Fr): Promise<void>;

  /**
   * Updates the latest global variables hash
   * @param globalVariablesHash - The latest global variables hash
   */
  updateLatestGlobalVariablesHash(globalVariablesHash: Fr): Promise<void>;

  /**
   * Gets the global variables hash from the previous block
   */
  getLatestGlobalVariablesHash(): Promise<Fr>;

  /**
   * Batch insert multiple leaves into the tree.
   * @param leaves - Leaves to insert into the tree.
   * @param treeId - The tree on which to insert.
   * @param subtreeHeight - Height of the subtree.
   * @returns The witness data for the leaves to be updated when inserting the new ones.
   */
  batchInsert(
    treeId: MerkleTreeId,
    leaves: Buffer[],
    subtreeHeight: number,
  ): Promise<[LowLeafWitnessData<number>[], SiblingPath<number>] | [undefined, SiblingPath<number>]>;

  /**
   * Handles a single L2 block (i.e. Inserts the new commitments into the merkle tree).
   * @param block - The L2 block to handle.
   */
  handleL2Block(block: L2Block): Promise<void>;

  /**
   * Commits pending changes to the underlying store.
   */
  commit(): Promise<void>;

  /**
   * Rolls back pending changes.
   */
  rollback(): Promise<void>;
}

/**
 * Outputs a tree leaves using for debugging purposes.
 */
export async function inspectTree(
  db: MerkleTreeOperations,
  treeId: MerkleTreeId,
  log = createDebugLogger('aztec:inspect-tree'),
) {
  const info = await db.getTreeInfo(treeId);
  const output = [`Tree id=${treeId} size=${info.size} root=0x${info.root.toString('hex')}`];
  for (let i = 0; i < info.size; i++) {
    output.push(
      ` Leaf ${i}: ${await db.getLeafValue(treeId, BigInt(i)).then(x => x?.toString('hex') ?? '[undefined]')}`,
    );
  }
  log(output.join('\n'));
}
