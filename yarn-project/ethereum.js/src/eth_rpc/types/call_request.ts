import { EthAddress } from '@aztec/foundation';
import {
  bigIntToHex,
  bufferToHex,
  hexToBigInt,
  hexToBuffer,
  hexToNumber,
  numberToHex,
} from '../../hex_string/index.js';

/**
 * Represents a call request object for Ethereum transactions.
 * Contains optional parameters such as 'from', 'to', 'gas', 'maxFeePerGas', 'maxPriorityFeePerGas', 'value', and 'data'.
 * Used to define the structure of the transaction details when making a function call or sending Ether between addresses.
 */
export interface CallRequest {
  /**
   * The sender's Ethereum address.
   */
  from?: EthAddress;
  /**
   * The destination Ethereum address for the call request.
   */
  to?: EthAddress;
  /**
   * The maximum amount of gas units to be used for the transaction execution.
   */
  gas?: number;
  /**
   * Maximum fee per gas unit for transaction processing.
   */
  maxFeePerGas?: bigint;
  /**
   * Maximum fee per gas for transaction priority.
   */
  maxPriorityFeePerGas?: bigint;
  /**
   * The amount of Ether (in wei) to be transferred with the call.
   */
  value?: bigint;
  /**
   * The input data to be executed by the called contract.
   */
  data?: Buffer;
}

/**
 * Represents a raw Ethereum call request object with fields in hexadecimal string format.
 * This interface is used to convert and interact with the CallRequest object, which uses
 * more specific data types like bigint and Buffer for better type safety and readability.
 */
export interface RawCallRequest {
  /**
   * The Ethereum address initiating the call.
   */
  from?: string;
  /**
   * The destination Ethereum address for the call request.
   */
  to?: string;
  /**
   * The maximum amount of gas allowed for executing the transaction.
   */
  gas?: string;
  /**
   * Maximum fee per gas unit for transaction processing.
   */
  maxFeePerGas?: string;
  /**
   * The maximum fee per gas unit prioritized for transaction inclusion.
   */
  maxPriorityFeePerGas?: string;
  /**
   * The amount of Ether to be transferred in the transaction, represented as a bigint.
   */
  value?: string;
  /**
   * The encoded function call data for the contract method.
   */
  data?: string;
}

/**
 * Convert a CallRequest object to its RawCallRequest representation.
 * This function takes a CallRequest object with typed properties (EthAddress, bigint, Buffer)
 * and returns a RawCallRequest object with the corresponding hex string representations.
 *
 * @param tx - The CallRequest object containing Ethereum transaction data in typed format.
 * @returns A RawCallRequest object with the same transaction data in hex string format.
 */
export function toRawCallRequest(tx: CallRequest): RawCallRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, data } = tx;
  return {
    from: from ? from.toString().toLowerCase() : undefined,
    to: to ? to.toString().toLowerCase() : undefined,
    data: data ? bufferToHex(data) : undefined,
    value: value ? bigIntToHex(value) : undefined,
    gas: gas ? numberToHex(gas) : undefined,
    maxFeePerGas: maxFeePerGas ? bigIntToHex(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? bigIntToHex(maxPriorityFeePerGas) : undefined,
  };
}

/**
 * Convert a RawCallRequest object into a CallRequest object by parsing and converting
 * its properties from hex-encoded strings to their respective data types.
 * This function handles 'from', 'to', 'gas', 'maxFeePerGas', 'maxPriorityFeePerGas',
 * 'value', and 'data' properties. It also creates EthAddress instances for 'from' and 'to'.
 * If any property is not present in the input, it will remain undefined in the output.
 *
 * @param tx - The RawCallRequest object with hex-encoded string properties.
 * @returns A CallRequest object with parsed and converted properties.
 */
export function fromRawCallRequest(tx: RawCallRequest): CallRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, data } = tx;
  return {
    from: from ? EthAddress.fromString(from) : undefined,
    to: to ? EthAddress.fromString(to) : undefined,
    data: data ? hexToBuffer(data) : undefined,
    value: value ? hexToBigInt(value) : undefined,
    gas: gas ? hexToNumber(gas) : undefined,
    maxFeePerGas: maxFeePerGas ? hexToBigInt(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? hexToBigInt(maxPriorityFeePerGas) : undefined,
  };
}
