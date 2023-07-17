import { CONTRACT_TREE_HEIGHT } from '@aztec/circuits.js';

import { SiblingPath } from '../sibling_path.js';

/**
 * Interface providing methods for retrieving information about a contract's location in the contract tree.
 */
export interface ContractCommitmentProvider {
  /**
   * Find the index of the given contract.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf in the contracts tree or undefined if not found.
   */
  findContractIndex(leafValue: Buffer): Promise<bigint | undefined>;

  /**
   * Returns the sibling path for the given index in the contract tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getContractPath(leafIndex: bigint): Promise<SiblingPath<typeof CONTRACT_TREE_HEIGHT>>;
}
