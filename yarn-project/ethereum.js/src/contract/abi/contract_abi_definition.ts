/**
 * Represents the supported data types in Ethereum ABI (Application Binary Interface) for encoding and decoding contract interactions.
 */
export type AbiDataTypes = 'bool' | 'string' | 'address' | 'function' | 'uint' | 'int' | 'bytes' | string;

/**
 * Type representing an individual input parameter in the ABI (Application Binary Interface) of a smart contract.
 * It includes properties for the input's name, data type, and other relevant information used in encoding/decoding
 * contract function calls and events.
 */
export type AbiInput = {
  /**
   * Represents the structure of nested tuple elements.
   */
  components?: any;
  /**
   * The name identifier for the contract entry.
   */
  name: string;
  /**
   * Represents the type of a Contract Entry in the ABI (Application Binary Interface) definition.
   */
  type: AbiDataTypes;
  /**
   * Indicates if the parameter is indexed in events.
   */
  indexed?: boolean;
  /**
   * The internal representation of the data type.
   */
  internalType?: string;
};

/**
 * Represents the type definition for a single output parameter in a contract's ABI.
 */
export type AbiOutput = {
  /**
   * Nested structure defining the data type components.
   */
  components?: any;
  /**
   * The name identifier of the contract entry.
   */
  name: string;
  /**
   * The type of contract entry, such as function, constructor, event, fallback, error, or receive.
   */
  type: AbiDataTypes;
  /**
   * Represents the internal Solidity type of the input/output.
   */
  internalType?: string;
};

/**
 * Represents a single entry in a smart contract's ABI definition.
 * Provides essential information about the contract's functions, events, constructors, and other elements,
 * allowing effective interaction with the Ethereum blockchain.
 */
export interface ContractEntryDefinition {
  /**
   * Indicates if the contract entry is constant (read-only).
   */
  constant?: boolean;
  /**
   * Indicates whether the contract entry can receive Ether.
   */
  payable?: boolean;
  /**
   * Indicates if the event is anonymous, omitting event signature from logs.
   */
  anonymous?: boolean;
  /**
   * An array of input parameters for the contract function or event.
   */
  inputs?: AbiInput[];
  /**
   * The identifier for the contract function, event, or variable.
   */
  name?: string;
  /**
   * An array of output parameters for the contract function or event.
   */
  outputs?: AbiOutput[];
  /**
   * The type of contract entry, representing its purpose and functionality.
   */
  type: 'function' | 'constructor' | 'event' | 'fallback' | 'error' | 'receive';
  /**
   * Represents the mutability of a contract's state during function execution.
   */
  stateMutability?: 'pure' | 'view' | 'payable' | 'nonpayable';
  /**
   * The unique function identifier generated from the function's name and input types.
   */
  signature?: string;
  /**
   * The estimated gas cost for executing the function.
   */
  gas?: number;
}

/**
 * Type representing the Application Binary Interface (ABI) definition for a smart contract,
 * which consists of an array of ContractEntryDefinition objects. The ABI defines the
 * structure of functions, events, and data types of a contract that can be interacted with.
 */
export type ContractAbiDefinition = ContractEntryDefinition[];
