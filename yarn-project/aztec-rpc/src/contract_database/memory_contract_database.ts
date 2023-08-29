import { FunctionSelector } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { DebugLogger } from '@aztec/foundation/log';
import { ContractDao, ContractDatabase } from '@aztec/types';

/**
 * The MemoryContractDatabase class serves as an in-memory implementation of the ContractDatabase interface.
 * It allows for storing and retrieving contract data, such as ContractDao objects and associated function bytecodes,
 * within a contracts array. This class is particularly useful for testing and development purposes where a
 * persistent storage may not be required.
 */
export class MemoryContractDatabase implements ContractDatabase {
  private contracts: ContractDao[] = [];

  constructor(protected log: DebugLogger) {}

  /**
   * Adds a new ContractDao instance to the memory-based contract database.
   * The function stores the contract in an array and returns a resolved promise indicating successful addition.
   *
   * @param contract - The ContractDao instance to be added to the memory database.
   * @returns A Promise that resolves when the contract is successfully added.
   */
  public addContract(contract: ContractDao) {
    this.log(`Adding contract ${contract.completeAddress.address.toString()}`);
    this.contracts.push(contract);
    return Promise.resolve();
  }

  /**
   * Retrieve a ContractDao instance with the specified AztecAddress from the in-memory contracts list.
   * Returns the first match found or undefined if no contract with the given address is found.
   *
   * @param address - The AztecAddress to search for in the stored contracts.
   * @returns A Promise resolving to the ContractDao instance matching the given address or undefined.
   */
  public getContract(address: AztecAddress): Promise<ContractDao | undefined> {
    return Promise.resolve(this.contracts.find(c => c.completeAddress.address.equals(address)));
  }

  public getContracts(): Promise<ContractDao[]> {
    return Promise.resolve(this.contracts);
  }

  /**
   * Retrieve the bytecode associated with a given contract address and function selector.
   * This function searches through the stored contracts to find a matching contract and function,
   * then returns the corresponding bytecode. If no match is found, it returns undefined.
   *
   * @param contractAddress - The AztecAddress representing the contract address to look for.
   * @param selector - The function selector.
   * @returns A Promise that resolves to the bytecode of the matching function or undefined if not found.
   */
  public async getCode(contractAddress: AztecAddress, selector: FunctionSelector) {
    const contract = await this.getContract(contractAddress);
    return contract?.functions.find(f => f.selector.equals(selector))?.bytecode;
  }
}
