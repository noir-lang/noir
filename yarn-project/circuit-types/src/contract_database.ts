import { AztecAddress } from '@aztec/foundation/aztec-address';

import { ContractDao } from './contract_dao.js';

/**
 * Represents a ContractDatabase interface for managing Aztec.nr contracts.
 * Provides methods for adding and retrieving ContractDao objects by their associated addresses.
 */
export interface ContractDatabase {
  /**
   * Adds a new ContractDao instance to the contract database.
   * The function stores the contract in an array and returns a resolved promise indicating successful addition.
   *
   * @param contract - The ContractDao instance to be added.
   * @returns A Promise that resolves when the contract is successfully added.
   */
  addContract(contract: ContractDao): Promise<void>;

  /**
   * Retrieve a ContractDao instance with the specified AztecAddress.
   * Returns the first match found or undefined if no contract with the given address is found.
   *
   * @param address - The AztecAddress to search for in the stored contracts.
   * @returns A Promise resolving to the ContractDao instance matching the given address or undefined.
   */
  getContract(address: AztecAddress): Promise<ContractDao | undefined>;

  /**
   * Retrieve all ContractDao instances stored in the database.
   * @returns A Promise resolving to an array of all stored ContractDao instances.
   */
  getContracts(): Promise<ContractDao[]>;
}
