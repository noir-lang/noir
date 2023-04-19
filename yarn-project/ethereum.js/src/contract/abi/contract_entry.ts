import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';

/**
 * The ContractEntry class represents a single entry within an Ethereum smart contract's ABI definition.
 * It provides easy access to the name of the function or event, as well as its anonymous status.
 * Additionally, it offers a method to convert the entry into a human-readable string format.
 * This class is primarily used for parsing and interacting with contract ABI definitions.
 */
export class ContractEntry {
  constructor(protected entry: ContractEntryDefinition) {}
  // eslint-disable-next-line jsdoc/require-jsdoc
  public get name() {
    return this.entry.name;
  }
  // eslint-disable-next-line jsdoc/require-jsdoc
  public get anonymous() {
    return this.entry.anonymous || false;
  }

  /**
   * Returns a string representation of the ContractEntry instance using ABI encoding.
   * This method utilizes the 'abiCoder' module to convert the contract entry definition
   * into a readable and formatted string.
   *
   * @returns A string representation of the ContractEntry instance with ABI encoding.
   */
  public asString() {
    return abiCoder.abiMethodToString(this.entry);
  }
}
