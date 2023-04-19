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
 * Represents a raw Ethereum transaction response.
 * Contains unprocessed data fields in hexadecimal string format, typically received from an Ethereum node API.
 */
export interface RawTransactionResponse {
  /**
   * The hash of the block containing the transaction.
   */
  blockHash: string | null;
  /**
   * The block number in which the transaction is included.
   */
  blockNumber: string | null;
  /**
   * The originating Ethereum address of the transaction.
   */
  from: string;
  /**
   * The amount of gas required for the transaction execution.
   */
  gas: string;
  /**
   * The price per unit of gas in the transaction.
   */
  gasPrice: string;
  /**
   * Maximum fee per gas unit for a transaction.
   */
  maxFeePerGas?: string;
  /**
   * The maximum fee per gas unit for transaction prioritization.
   */
  maxPriorityFeePerGas?: string;
  /**
   * The unique identifier of the transaction.
   */
  hash: string;
  /**
   * Raw input data of the transaction.
   */
  input: string;
  /**
   * A unique transaction counter for the sender.
   */
  nonce: string;
  /**
   * The destination Ethereum address involved in the transaction.
   */
  to: string | null;
  /**
   * The index of the transaction within its containing block.
   */
  transactionIndex: string | null;
  /**
   * The Ethereum transaction type identifier.
   */
  type: string;
  /**
   * The amount of Ether transferred in the transaction.
   */
  value: string;
  /**
   * The recovery identifier of the transaction signature.
   */
  v: string;
  /**
   * The 'r' value of the transaction's ECDSA signature.
   */
  r: string;
  /**
   * Signature component for transaction verification.
   */
  s: string;
}

/**
 * Represents an Ethereum transaction response with decoded data types.
 * Provides a structured interface for working with transaction responses returned from the Ethereum network.
 */
export interface TransactionResponse {
  /**
   * The hash of the block containing the transaction.
   */
  blockHash: string | null;
  /**
   * The block number containing the transaction, or null if not yet mined.
   */
  blockNumber: number | null;
  /**
   * The originating Ethereum address of the transaction.
   */
  from: EthAddress;
  /**
   * Amount of gas units required for executing the transaction.
   */
  gas: number;
  /**
   * The amount of Ether paid per unit of gas for the transaction.
   */
  gasPrice: bigint;
  /**
   * The maximum fee per gas unit for the transaction.
   */
  maxFeePerGas?: bigint;
  /**
   * The maximum fee per gas a user is willing to pay for transaction priority.
   */
  maxPriorityFeePerGas?: bigint;
  /**
   * The unique identifier of the transaction.
   */
  hash: string;
  /**
   * Raw binary data representing smart contract method calls and parameters.
   */
  input: Buffer;
  /**
   * An integer value representing the number of transactions sent by the sender.
   */
  nonce: number;
  /**
   * The destination Ethereum address for the transaction.
   */
  to: EthAddress | null;
  /**
   * The position of the transaction within the block.
   */
  transactionIndex: number | null;
  /**
   * Transaction type identifier.
   */
  type: number;
  /**
   * The value transferred in the transaction, represented as a bigint.
   */
  value: bigint;
  /**
   * The recovery identifier of the ECDSA signature.
   */
  v: string;
  /**
   * The 'r' value of the ECDSA signature.
   */
  r: string;
  /**
   * Signature recovery value for ECDSA.
   */
  s: string;
}

/**
 * Converts a raw transaction response object into a more structured and typed TransactionResponse object.
 * This function decodes hex-encoded strings, converts number values to their respective types (BigInt or Number),
 * and represents Ethereum addresses as EthAddress instances. It can handle both EIP-1559 and legacy transactions.
 *
 * @param tx - The raw transaction response object, typically retrieved from an Ethereum node API.
 * @returns A structured and typed TransactionResponse object with decoded values for easier handling and manipulation.
 */
export function fromRawTransactionResponse(tx: RawTransactionResponse): TransactionResponse {
  return {
    ...tx,
    blockNumber: tx.blockNumber ? hexToNumber(tx.blockNumber) : null,
    transactionIndex: tx.transactionIndex ? hexToNumber(tx.transactionIndex) : null,
    nonce: hexToNumber(tx.nonce),
    gas: hexToNumber(tx.gas),
    gasPrice: hexToBigInt(tx.gasPrice),
    maxFeePerGas: tx.maxFeePerGas ? hexToBigInt(tx.gasPrice) : undefined,
    maxPriorityFeePerGas: tx.maxPriorityFeePerGas ? hexToBigInt(tx.gasPrice) : undefined,
    value: hexToBigInt(tx.value),
    type: hexToNumber(tx.type),
    to: tx.to ? EthAddress.fromString(tx.to) : null,
    from: EthAddress.fromString(tx.from),
    input: hexToBuffer(tx.input),
  };
}

/**
 * Converts a TransactionResponse object into a RawTransactionResponse object by transforming its field values to their raw string or number format.
 * This function is useful when dealing with APIs or systems that expect transaction data in raw string or number format.
 *
 * @param tx - The TransactionResponse object containing the transaction data in bigint, Buffer, or EthAddress format.
 * @returns A RawTransactionResponse object containing the transaction data in raw string or number format.
 */
export function toRawTransactionResponse(tx: TransactionResponse): RawTransactionResponse {
  return {
    ...tx,
    blockNumber: tx.blockNumber ? numberToHex(tx.blockNumber) : null,
    transactionIndex: tx.transactionIndex ? numberToHex(tx.transactionIndex) : null,
    nonce: numberToHex(tx.nonce)!,
    gas: numberToHex(tx.gas)!,
    gasPrice: bigIntToHex(tx.gasPrice),
    maxFeePerGas: tx.maxFeePerGas ? bigIntToHex(tx.maxFeePerGas) : undefined,
    maxPriorityFeePerGas: tx.maxPriorityFeePerGas ? bigIntToHex(tx.maxPriorityFeePerGas) : undefined,
    value: bigIntToHex(tx.value),
    type: numberToHex(tx.type),
    to: tx.to ? tx.to.toString() : null,
    from: tx.from.toString(),
    input: bufferToHex(tx.input),
  };
}
