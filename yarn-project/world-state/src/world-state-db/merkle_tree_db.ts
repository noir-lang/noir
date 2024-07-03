import { type MerkleTreeId } from '@aztec/circuit-types';
import { type Fr, MAX_NULLIFIERS_PER_TX, MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX } from '@aztec/circuits.js';
import { type IndexedTreeSnapshot, type TreeSnapshot } from '@aztec/merkle-tree';

import { type MerkleTreeOperations } from './merkle_tree_operations.js';

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
export const INITIAL_NULLIFIER_TREE_SIZE = 2 * MAX_NULLIFIERS_PER_TX;

export const INITIAL_PUBLIC_DATA_TREE_SIZE = 2 * MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX;

/**
 * Adds a last boolean flag in each function on the type.
 */
type WithIncludeUncommitted<F> = F extends (...args: [...infer Rest]) => infer Return
  ? (...args: [...Rest, boolean]) => Return
  : F;

/**
 * Defines the names of the setters on Merkle Trees.
 */
type MerkleTreeSetters =
  | 'appendLeaves'
  | 'updateLeaf'
  | 'commit'
  | 'rollback'
  | 'handleL2BlockAndMessages'
  | 'batchInsert';

export type TreeSnapshots = {
  [MerkleTreeId.NULLIFIER_TREE]: IndexedTreeSnapshot;
  [MerkleTreeId.NOTE_HASH_TREE]: TreeSnapshot<Fr>;
  [MerkleTreeId.PUBLIC_DATA_TREE]: IndexedTreeSnapshot;
  [MerkleTreeId.L1_TO_L2_MESSAGE_TREE]: TreeSnapshot<Fr>;
  [MerkleTreeId.ARCHIVE]: TreeSnapshot<Fr>;
};

/**
 * Defines the interface for operations on a set of Merkle Trees configuring whether to return committed or uncommitted data.
 */
export type MerkleTreeDb = {
  [Property in keyof MerkleTreeOperations as Exclude<Property, MerkleTreeSetters>]: WithIncludeUncommitted<
    MerkleTreeOperations[Property]
  >;
} & Pick<MerkleTreeOperations, MerkleTreeSetters> & {
    /**
     * Returns a snapshot of the current state of the trees.
     * @param block - The block number to take the snapshot at.
     */
    getSnapshot(block: number): Promise<TreeSnapshots>;
  };
