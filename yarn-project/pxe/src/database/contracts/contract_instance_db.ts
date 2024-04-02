import { type AztecAddress } from '@aztec/circuits.js';
import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

/**
 * PXE database for managing contract instances.
 */
export interface ContractInstanceDatabase {
  /**
   * Adds a new contract to the db or updates an existing one.
   * @param contract - Contract to insert.
   */
  addContractInstance(contract: ContractInstanceWithAddress): Promise<void>;
  /**
   * Gets a contract given its address.
   * @param address - Address of the contract.
   */
  getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined>;

  /** Returns the addresses all contract instances registered in the DB. */
  getContractsAddresses(): Promise<AztecAddress[]>;
}
