import { hexToBuffer } from '../../hex_string/index.js';
import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';
import { ContractEntry } from './contract_entry.js';

/**
 * The ContractErrorEntry class extends the functionalities of the ContractEntry class for error handling in smart contracts.
 * It handles encoding, decoding and managing error entries in a contract's ABI (Application Binary Interface).
 * This class provides methods to encode and decode parameters, return values, and ABI for contract errors, ensuring proper communication with the blockchain.
 */
export class ContractErrorEntry extends ContractEntry {
  /**
   * The encoded function signature for the contract entry.
   */
  public readonly signature: Buffer;

  constructor(entry: ContractEntryDefinition) {
    entry.inputs = entry.inputs || [];
    super(entry);
    this.signature = hexToBuffer(abiCoder.encodeFunctionSignature(abiCoder.abiMethodToString(entry)));
  }

  /**
   * Retrieve the number of input arguments for this contract error entry.
   * This function returns the length of the 'inputs' array, which represents
   * the input arguments required by the entry. If no inputs are defined,
   * it returns 0.
   *
   * @returns The number of input arguments for the contract error entry.
   */
  public numArgs() {
    return this.entry.inputs ? this.entry.inputs.length : 0;
  }

  /**
   * Decodes the return value of a contract function call using the ABI output definition.
   * If there is only one output, returns the decoded output value directly; otherwise,
   * returns an object containing the decoded values with the output names as keys.
   * If the input returnValue buffer is empty, returns null.
   *
   * @param returnValue - The Buffer containing the encoded return value of the contract function call.
   * @returns Decoded output value(s) or null if returnValue is empty.
   */
  public decodeReturnValue(returnValue: Buffer) {
    if (!returnValue.length) {
      return null;
    }

    const result = abiCoder.decodeParameters(this.entry.outputs, returnValue);

    if (result.__length__ === 1) {
      return result[0];
    } else {
      delete result.__length__;
      return result;
    }
  }

  /**
   * Encodes the ABI (Application Binary Interface) of a function call by concatenating the function's signature
   * and encoded input parameters. This resulting buffer can be used for encoding the data field of a transaction.
   * The 'args' array should contain values that match the expected input types of the function.
   *
   * @param args - An array of arguments matching the function's input parameters.
   * @returns A Buffer containing the encoded ABI for the function call.
   */
  public encodeABI(args: any[]) {
    return Buffer.concat([this.signature, this.encodeParameters(args)]);
  }

  /**
   * Encode the input parameters according to the contract entry inputs.
   * This function takes an array of arguments and encodes them into a Buffer
   * following the Solidity contract's entry ABI specifications.
   *
   * @param args - An array of input values matching the contract entry inputs.
   * @returns A Buffer containing the encoded parameters.
   */
  public encodeParameters(args: any[]) {
    return abiCoder.encodeParameters(this.entry.inputs, args);
  }

  /**
   * Decode the provided bytes buffer into parameters based on the entry inputs.
   * This function helps in interpreting the raw bytes buffer received from a contract call
   * or an event log, by decoding it based on the ABI input types, and returning the
   * decoded values as an object with the input names as keys.
   *
   * @param bytes - The Buffer containing the encoded parameters to be decoded.
   * @returns An object with decoded parameters, keys mapped to the input names defined in the ABI.
   */
  public decodeParameters(bytes: Buffer) {
    return abiCoder.decodeParameters(this.entry.inputs, bytes);
  }
}
