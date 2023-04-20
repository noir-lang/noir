import { LeafData } from '../index.js';
import { MerkleTree } from './merkle_tree.js';

export interface UpdateOnlyTree extends MerkleTree {
  /**
   * Updates a leaf at a given index in the tree
   * @param leaf The leaf value to be updated
   * @param index The leaf to be updated
   */
  // TODO: Make this strictly a Buffer
  updateLeaf(leaf: Buffer | LeafData, index: bigint): Promise<void>;
}
