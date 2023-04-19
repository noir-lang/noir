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
 * Represents an Ethereum transaction request.
 * Provides a structured format for specifying the required parameters when sending a new transaction on the Ethereum network.
 */
export interface TransactionRequest {
  /**
   * The Ethereum address initiating the transaction.
   */
  from: EthAddress;
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
   * The maximum fee per gas unit that the sender is willing to pay for transaction priority.
   */
  maxPriorityFeePerGas?: bigint;
  /**
   * The amount of Ether to be transferred in the transaction.
   */
  value?: bigint;
  /**
   * The encoded contract function call data.
   */
  data?: Buffer;
  /**
   * A unique number that prevents double-spending in transactions.
   */
  nonce?: number;
}

/**
 * Type representing a raw Ethereum transaction request, where all properties are encoded as hexadecimal strings.
 * Useful for serialization and deserialization of transaction data to and from external systems or storage.
 */
export type RawTransactionRequest = { [k in keyof TransactionRequest]: string };

/**
 * Converts a TransactionRequest object into a RawTransactionRequest object by converting all properties to string format.
 * The function ensures that the 'from' and 'to' addresses are in lowercase, and bigint, number, and Buffer values are converted to hexadecimal strings.
 * If a property is not present in the input TransactionRequest object, it will be set to 'undefined' in the output RawTransactionRequest object.
 *
 * @param tx - The TransactionRequest object to be converted.
 * @returns A RawTransactionRequest object with all properties in string format.
 */
export function toRawTransactionRequest(tx: TransactionRequest): RawTransactionRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, nonce, data } = tx;
  return {
    from: from.toString().toLowerCase(),
    to: to ? to.toString().toLowerCase() : undefined,
    gas: gas ? numberToHex(gas) : undefined,
    value: value ? bigIntToHex(value) : undefined,
    data: data ? bufferToHex(data) : undefined,
    nonce: nonce ? numberToHex(nonce) : undefined,
    maxFeePerGas: maxFeePerGas ? bigIntToHex(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? bigIntToHex(maxPriorityFeePerGas) : undefined,
  };
}

/**
 * Convert a raw transaction request object with its properties as hex-encoded strings into a TransactionRequest object
 * with the corresponding native JavaScript types. This function is useful for decoding raw transaction requests received
 * from external sources, such as APIs or user inputs, and converting them into a format suitable for further processing.
 *
 * @param tx - The raw transaction request object with its properties as hex-encoded strings.
 * @returns A TransactionRequest object with the corresponding native JavaScript types.
 */
export function fromRawTransactionRequest(tx: RawTransactionRequest): TransactionRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, nonce, data } = tx;
  return {
    from: EthAddress.fromString(from),
    to: to ? EthAddress.fromString(to) : undefined,
    gas: gas ? hexToNumber(gas) : undefined,
    value: value ? hexToBigInt(value) : undefined,
    data: data ? hexToBuffer(data) : undefined,
    nonce: nonce ? hexToNumber(nonce) : undefined,
    maxFeePerGas: maxFeePerGas ? hexToBigInt(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? hexToBigInt(maxPriorityFeePerGas) : undefined,
  };
}
