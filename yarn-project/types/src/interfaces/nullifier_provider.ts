import { Fr } from '@aztec/foundation/fields';

/**
 * Interface for providing information about nullifiers within the nullifier tree.
 */
export interface NullifierProvider {
  /**
   * Find the index of the given nullifier.
   * @param nullifier - The nullifier to search for.
   * @returns The index of the given leaf of undefined if not found.
   */
  findNullifierIndex(nullifier: Fr): Promise<bigint | undefined>;
}
