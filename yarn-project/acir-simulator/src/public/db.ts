import { EthAddress } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

/**
 * Database interface for providing access to public state.
 */
export interface PublicStateDB {
  /**
   * Reads a value from public storage, returning zero if none.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @returns The current value in the storage slot.
   */
  storageRead(contract: AztecAddress, slot: Fr): Promise<Fr>;
}

/**
 * Database interface for providing access to public contract data.
 */
export interface PublicContractsDB {
  /**
   * Returns the brillig (public bytecode) of a function.
   * @param address - The contract address that owns this function.
   * @param functionSelector - The selector for the function.
   * @returns The bytecode or undefined if not found.
   */
  getBytecode(address: AztecAddress, functionSelector: Buffer): Promise<Buffer | undefined>;

  /**
   * Returns the portal contract address for an L2 address.
   * @param address - The L2 contract address.
   * @returns The portal contract address or undefined if not found.
   */
  getPortalContractAddress(address: AztecAddress): Promise<EthAddress | undefined>;
}
