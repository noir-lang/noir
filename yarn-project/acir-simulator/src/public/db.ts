import { AztecAddress, Fr } from '@aztec/foundation';

export interface PublicDB {
  /**
   * Reads a value from public storage, returning zero if none.
   * @param contract - owner of the storage.
   * @param slot - slot to read in the contract storage.
   * @returns The current value in the storage slot.
   */
  storageRead(contract: AztecAddress, slot: Fr): Promise<Fr>;
}
