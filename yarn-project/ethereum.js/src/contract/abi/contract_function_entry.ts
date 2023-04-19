import { hexToBuffer } from '../../hex_string/index.js';
import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';
import { ContractEntry } from './contract_entry.js';

/**
 * The ContractFunctionEntry class represents a function entry within a smart contract ABI definition.
 * It provides methods for encoding and decoding parameters, as well as determining the function's constant and payable properties.
 * This class extends the ContractEntry base class and adds functionality specific to smart contract functions such as constructors and regular methods.
 */
export class ContractFunctionEntry extends ContractEntry {
  /**
   * The unique identifier of the contract function.
   */
  public readonly signature: string;

  constructor(entry: ContractEntryDefinition) {
    entry.inputs = entry.inputs || [];
    super(entry);
    this.signature =
      entry.type === 'constructor'
        ? 'constructor'
        : abiCoder.encodeFunctionSignature(abiCoder.abiMethodToString(entry));
  }
  // eslint-disable-next-line jsdoc/require-jsdoc
  public get constant() {
    return this.entry.stateMutability === 'view' || this.entry.stateMutability === 'pure' || this.entry.constant;
  }
  // eslint-disable-next-line jsdoc/require-jsdoc
  public get payable() {
    return this.entry.stateMutability === 'payable' || this.entry.payable;
  }

  /**
   * Returns the number of input arguments required for the contract function.
   * This value is derived from the 'inputs' property of the contract entry definition.
   *
   * @returns The number of input arguments required for the function.
   */
  public numArgs() {
    return this.entry.inputs ? this.entry.inputs.length : 0;
  }

  /**
   * Decodes the return value of a contract function call.
   * This method takes a Buffer containing the raw return value from a contract function call
   * and decodes it according to the output parameters defined in the ABI.
   * If the decoded result contains only one value, it returns that value directly;
   * otherwise, an object with named properties is returned, excluding the '__length__' property.
   *
   * @param returnValue - The raw return value from a contract function call as a Buffer.
   * @returns The decoded value(s) according to the output parameters defined in the ABI,
   * either as a single value or an object with named properties.
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
   * Encodes the function call and its arguments into ABI format (Application Binary Interface).
   * This representation is used for interacting with the Ethereum blockchain.
   * The encoded result is a Buffer that can be sent as data in a transaction or used to invoke contract functions.
   *
   * @param args - An array of values representing the arguments to pass in the function call.
   * @returns A Buffer containing the encoded function signature and parameters in ABI format.
   */
  public encodeABI(args: any[]) {
    return Buffer.concat([hexToBuffer(this.signature), this.encodeParameters(args)]);
  }

  /**
   * Encode the provided arguments based on the contract function's input parameters.
   * This is useful when preparing ABI-encoded data to interact with a smart contract function.
   * Throws an error if the provided arguments don't match the expected input parameters.
   *
   * @param args - An array of values representing the arguments for the contract function.
   * @returns A Buffer containing the ABI-encoded parameters.
   */
  public encodeParameters(args: any[]) {
    return abiCoder.encodeParameters(this.entry.inputs, args);
  }

  /**
   * Decode the parameters from a given buffer using the input types defined in the contract entry.
   * This function is useful for unpacking parameters from encoded data or transaction payloads.
   *
   * @param bytes - The buffer containing the encoded parameters.
   * @returns An object with the decoded parameters mapped to their respective names as defined in the contract entry.
   */
  public decodeParameters(bytes: Buffer) {
    return abiCoder.decodeParameters(this.entry.inputs, bytes);
  }
}
