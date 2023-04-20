import { LeafData, SiblingPath } from '@aztec/merkle-tree';

export * from './merkle_trees.js';
export { LeafData } from '@aztec/merkle-tree';

/**
 * Defines the possible Merkle tree IDs.
 */
export enum MerkleTreeId {
  CONTRACT_TREE = 0,
  CONTRACT_TREE_ROOTS_TREE = 1,
  NULLIFIER_TREE = 2,
  PRIVATE_DATA_TREE = 3,
  PRIVATE_DATA_TREE_ROOTS_TREE = 4,
  PUBLIC_DATA_TREE = 5,
}

export type IndexedTreeId = MerkleTreeId.NULLIFIER_TREE;
export type PublicTreeId = MerkleTreeId.PUBLIC_DATA_TREE;

/**
 * The nullifier tree must be pre filled with the number of leaves that are added by one rollup.
 * The tree must be initially padded as the pre-populated 0 index prevents efficient subtree insertion.
 * Padding with some values solves this issue.
 */
export const INITIAL_NULLIFIER_TREE_SIZE = 8;

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

type MerkleTreeSetters = 'appendLeaves';

/**
 * Defines the interface for operations on a set of Merkle Trees configuring whether to return committed or uncommitted data.
 */
export type MerkleTreeDbOperations = {
  [Property in keyof MerkleTreeOperations as Exclude<Property, MerkleTreeSetters>]: WithIncludeUncommitted<
    MerkleTreeOperations[Property]
  >;
} & Pick<MerkleTreeOperations, MerkleTreeSetters>;

/**
 * Defines the interface for operations on a set of Merkle Trees.
 */
export interface MerkleTreeOperations {
  /**
   * Appends leaves to a given tree
   * @param treeId - The tree to be updated
   * @param leaves - The set of leaves to be appended
   */
  appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void>;
  /**
   * Returns information about the given tree
   * @param treeId - The tree to be queried
   */
  getTreeInfo(treeId: MerkleTreeId): Promise<TreeInfo>;
  /**
   * Gets sibling path for a leaf.
   * @param treeId - The tree to be queried for a sibling path
   * @param index - The index of the leaf for which a sibling path should be returned
   */
  getSiblingPath(treeId: MerkleTreeId, index: bigint): Promise<SiblingPath>;
  /**
   * Returns the previous index for a given value in an indexed tree
   * @param treeId - The tree for which the previous value index is required
   * @param value - The value to be queried
   */
  getPreviousValueIndex(treeId: IndexedTreeId, value: bigint): Promise<{ index: number; alreadyPresent: boolean }>;
  /**
   * Returns the data at a specific leaf
   * @param treeId - The tree for which leaf data should be returned
   * @param index - The index of the leaf required
   */
  getLeafData(treeId: IndexedTreeId, index: number): Promise<LeafData | undefined>;
  /**
   * Update the leaf data at the given index
   * @param treeId - The tree for which leaf data should be edited
   * @param leaf - The updated leaf value
   * @param index - The index of the leaf to be updated
   */
  updateLeaf(treeId: IndexedTreeId | PublicTreeId, leaf: LeafData | Buffer, index: bigint): Promise<void>;
  /**
   * Returns the index containing a leaf value
   * @param treeId - The tree for which the index should be returned
   * @param value - The value to search for in the tree
   */
  findLeafIndex(treeId: MerkleTreeId, value: Buffer): Promise<bigint | undefined>;
  /**
   * Gets the value for a leaf in the tree.
   * @param treeId - The tree for which the index should be returned
   * @param index - The index of the leaf
   */
  getLeafValue(treeId: MerkleTreeId, index: bigint): Promise<Buffer | undefined>;
}

/**
 * Defines the interface for a database that stores Merkle trees.
 */
export interface MerkleTreeDb extends MerkleTreeDbOperations {
  /**
   * Commits pending changes to the underlying store
   */
  commit(): Promise<void>;
  /**
   * Rolls back pending changes
   */
  rollback(): Promise<void>;
}

/**
 * Outputs a tree leaves to console.log for debugging purposes.
 */
export async function inspectTree(db: MerkleTreeOperations, treeId: MerkleTreeId) {
  const info = await db.getTreeInfo(treeId);
  const output = [`Tree id=${treeId} size=${info.size} root=0x${info.root.toString('hex')}`];
  for (let i = 0; i < info.size; i++) {
    output.push(
      ` Leaf ${i}: ${await db.getLeafValue(treeId, BigInt(i)).then(x => x?.toString('hex') ?? '[undefined]')}`,
    );
  }
  console.log(output.join('\n'));
}
