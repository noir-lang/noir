import { EthAddress } from '@aztec/foundation';
import { fromRawLogResponse, LogResponse, RawLogResponse, toRawLogResponse } from './log_response.js';
import { numberToHex } from '../../hex_string/index.js';
import { TxHash } from '../tx_hash.js';

/**
 * Represents a raw Ethereum transaction receipt object.
 * Contains essential information about the transaction, including its hash, index, block number, and gas usage.
 * Also includes details about the sender, recipient, contract address, logs, and the transaction's status.
 */
export interface RawTransactionReceipt {
  /**
   * The unique identifier of the transaction.
   */
  transactionHash: string;
  /**
   * The index of the transaction within the block.
   */
  transactionIndex: string;
  /**
   * The hash identifier of the block containing the transaction.
   */
  blockHash: string;
  /**
   * The block number in which the transaction was included.
   */
  blockNumber: string;
  /**
   * The Ethereum address of the transaction sender.
   */
  from: string;
  /**
   * The destination Ethereum address involved in the transaction.
   */
  to: string | null;
  /**
   * The total amount of gas used by all transactions in the block up to and including this transaction.
   */
  cumulativeGasUsed: string;
  /**
   * The amount of gas consumed by the transaction.
   */
  gasUsed: string;
  /**
   * Address of the deployed contract, if applicable.
   */
  contractAddress: string | null;
  /**
   * An array of event logs emitted by the smart contract during the transaction execution.
   */
  logs: RawLogResponse[];
  /**
   * The transaction success status, where 'true' indicates success and 'false' indicates failure.
   */
  status: string;
}

/**
 * Represents a transaction receipt on the Ethereum blockchain.
 * Contains detailed information about a processed transaction, including its status, gas usage, and associated logs.
 */
export interface TransactionReceipt {
  /**
   * The unique hash identifier of the transaction.
   */
  transactionHash: TxHash;
  /**
   * The index of the transaction within its containing block.
   */
  transactionIndex: number;
  /**
   * The unique identifier of the block containing the transaction.
   */
  blockHash: string;
  /**
   * The block number containing the transaction.
   */
  blockNumber: number;
  /**
   * The Ethereum address of the transaction sender.
   */
  from: EthAddress;
  /**
   * The destination Ethereum address involved in the transaction.
   */
  to?: EthAddress;
  /**
   * The total amount of gas used by all transactions up to and including this one in the block.
   */
  cumulativeGasUsed: number;
  /**
   * The amount of gas utilized during the transaction execution.
   */
  gasUsed: number;
  /**
   * The Ethereum address of the deployed smart contract, if applicable.
   */
  contractAddress?: EthAddress;
  /**
   * An array of log events emitted by the transaction.
   */
  logs: LogResponse[];
  /**
   * The transaction execution status; true if successful, false otherwise.
   */
  status: boolean;
}

/**
 * Converts a RawTransactionReceipt object to a TransactionReceipt object.
 * Transforms string representations of properties such as transaction index, block number,
 * cumulative gas used, and gas used into their respective numeric types. Additionally,
 * it converts the 'from', 'to', and 'contractAddress' properties to EthAddress instances,
 * and the 'logs' property to LogResponse objects. The function also converts the 'status'
 * property to a boolean value, indicating the success or failure of the transaction.
 *
 * @param receipt - The RawTransactionReceipt object to be converted.
 * @returns A TransactionReceipt object with transformed properties or undefined if the input receipt is invalid or missing.
 */
export function fromRawTransactionReceipt(receipt?: RawTransactionReceipt): TransactionReceipt | undefined {
  if (!receipt || !receipt.blockHash) {
    return;
  }

  return {
    ...receipt,
    to: receipt.to ? EthAddress.fromString(receipt.to) : undefined,
    from: EthAddress.fromString(receipt.from),
    blockNumber: Number(receipt.blockNumber),
    transactionIndex: Number(receipt.transactionIndex),
    transactionHash: TxHash.fromString(receipt.transactionHash),
    cumulativeGasUsed: Number(receipt.cumulativeGasUsed),
    gasUsed: Number(receipt.gasUsed),
    logs: receipt.logs.map(fromRawLogResponse),
    contractAddress: receipt.contractAddress ? EthAddress.fromString(receipt.contractAddress) : undefined,
    status: Boolean(Number(receipt.status)),
  };
}

/**
 * Converts a TransactionReceipt object to its raw form, which is an object containing string representations of all properties.
 * The function takes care of converting the properties such as EthAddress objects, numbers, and booleans to their corresponding
 * string forms. Useful for serialization or converting the data back to a format that can be sent over the network or stored.
 *
 * @param receipt - The TransactionReceipt object to be converted to its raw form.
 * @returns A RawTransactionReceipt object containing string representations of all properties from the input TransactionReceipt.
 */
export function toRawTransactionReceipt(receipt: TransactionReceipt): RawTransactionReceipt {
  return {
    ...receipt,
    to: receipt.to ? receipt.to.toString() : null,
    from: receipt.from.toString(),
    blockNumber: numberToHex(receipt.blockNumber),
    transactionIndex: numberToHex(receipt.transactionIndex),
    transactionHash: receipt.transactionHash.toString(),
    cumulativeGasUsed: numberToHex(receipt.cumulativeGasUsed),
    gasUsed: numberToHex(receipt.gasUsed)!,
    logs: receipt.logs.map(toRawLogResponse),
    contractAddress: receipt.contractAddress ? receipt.contractAddress.toString() : null,
    status: numberToHex(receipt.status ? 1 : 0),
  };
}
