import { PRIVATE_DATA_TREE_HEIGHT } from '@aztec/circuits.js';

import { SiblingPath } from '../sibling_path.js';

/**
 * Interface for providing information about commitments within the data tree.
 */
export interface DataCommitmentProvider {
  /**
   * Find the index of the given commitment.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf of undefined if not found.
   */
  findCommitmentIndex(leafValue: Buffer): Promise<bigint | undefined>;

  /**
   * Returns the sibling path for the given index in the data tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getDataTreePath(leafIndex: bigint): Promise<SiblingPath<typeof PRIVATE_DATA_TREE_HEIGHT>>;
}
