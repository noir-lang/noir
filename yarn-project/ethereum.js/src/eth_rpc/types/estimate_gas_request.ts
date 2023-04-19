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
 * Represents an Ethereum transaction estimate request.
 * Contains information about the sender, receiver, gas limit, and other transaction-related details
 * necessary for estimating the gas cost of a transaction on the Ethereum network.
 */
export interface EstimateRequest {
  /**
   * The Ethereum address of the transaction sender.
   */
  from?: EthAddress;
  /**
   * The destination Ethereum address for the transaction.
   */
  to?: EthAddress;
  /**
   * The maximum amount of gas units allowed for the transaction execution.
   */
  gas?: number;
  /**
   * The maximum fee per gas unit for the transaction.
   */
  maxFeePerGas?: bigint;
  /**
   * The maximum fee per gas unit for transaction prioritization.
   */
  maxPriorityFeePerGas?: bigint;
  /**
   * The amount of Ether to be sent in the transaction, represented as a bigint.
   */
  value?: bigint;
  /**
   * The byte array of the transaction's input data.
   */
  data?: Buffer;
}

/**
 * Represents a raw estimate request object for Ethereum transactions.
 * Contains essential transaction parameters in hexadecimal string format.
 * This format is commonly used when interacting with Ethereum nodes or web3.js library.
 */
export interface RawEstimateRequest {
  /**
   * The Ethereum address initiating the transaction.
   */
  from?: string;
  /**
   * The destination Ethereum address for the transaction.
   */
  to?: string;
  /**
   * The maximum amount of gas units to be used for the transaction.
   */
  gas?: string;
  /**
   * Maximum fee per gas unit for the transaction.
   */
  maxFeePerGas?: string;
  /**
   * The maximum fee per gas unit to prioritize transaction processing.
   */
  maxPriorityFeePerGas?: string;
  /**
   * The amount of Ether to be transferred in the transaction, represented as a BigInt.
   */
  value?: string;
  /**
   * The transaction's input data as a Buffer.
   */
  data?: string;
}

/**
 * Converts an EstimateRequest object into a RawEstimateRequest object by transforming its properties.
 * This function is useful for preparing an EstimateRequest to be sent over RPC or other serialization protocols.
 * It converts EthAddress instances to strings, BigInt values to hex-encoded strings, and Buffers to hex-encoded strings.
 *
 * @param tx - The EstimateRequest object containing transaction properties.
 * @returns A RawEstimateRequest object with transformed properties.
 */
export function toRawEstimateRequest(tx: EstimateRequest): RawEstimateRequest {
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
 * Converts a RawEstimateRequest object with hex-encoded strings to an EstimateRequest object
 * with appropriate data types, such as BigInt and Buffer. This is useful when working with
 * Ethereum transaction estimates received from external sources in a raw format.
 *
 * @param tx - The RawEstimateRequest object with hex-encoded string values.
 * @returns An EstimateRequest object with appropriate data types.
 */
export function fromRawEstimateRequest(tx: RawEstimateRequest): EstimateRequest {
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
