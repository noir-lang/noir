import { keccak256String } from '../../crypto/index.js';
import { EthAddress } from '@aztec/foundation';
import { numberToHex } from '../../hex_string/index.js';
import { TxHash } from '../tx_hash.js';

/**
 * Represents the raw log response data structure from an Ethereum node.
 * Contains information about events emitted during the execution of a smart contract transaction.
 */
export interface RawLogResponse {
  /**
   * Unique identifier for a log event.
   */
  id?: string;
  /**
   * Indicates if the log entry has been removed due to a chain reorganization.
   */
  removed?: boolean;
  /**
   * The index of the log entry within its corresponding block.
   */
  logIndex: string | null;
  /**
   * The block number where the log was generated.
   */
  blockNumber: string | null;
  /**
   * The unique identifier (hash) of the block containing the log event.
   */
  blockHash: string | null;
  /**
   * The unique hash identifying a specific transaction.
   */
  transactionHash: string | null;
  /**
   * The index position of the transaction within a block.
   */
  transactionIndex: string | null;
  /**
   * The Ethereum address associated with the log event.
   */
  address: string;
  /**
   * The hexadecimal encoded data associated with the log event.
   */
  data: string;
  /**
   * An array of indexed event parameters.
   */
  topics: string[];
}

/**
 * Represents a log response from the Ethereum network.
 * Contains information about events emitted by smart contracts during transaction execution,
 * such as event signatures, topics, and data associated with the event.
 */
export interface LogResponse {
  /**
   * Unique identifier for the log event.
   */
  id: string | null;
  /**
   * Indicates whether the log entry has been removed due to a chain reorganization.
   */
  removed?: boolean;
  /**
   * The index position of the log entry within the block.
   */
  logIndex: number | null;
  /**
   * The block number in which the log was generated.
   */
  blockNumber: number | null;
  /**
   * The unique hash identifier of the block containing the log entry.
   */
  blockHash: string | null;
  /**
   * The unique identifier of the transaction in the blockchain.
   */
  transactionHash: TxHash | null;
  /**
   * The index position of the transaction within the block.
   */
  transactionIndex: number | null;
  /**
   * The Ethereum address associated with the log event.
   */
  address: EthAddress;
  /**
   * The data field of a logged event in the Ethereum contract.
   */
  data: string;
  /**
   * An array of indexed event arguments.
   */
  topics: string[];
}

/**
 * Converts a RawLogResponse object into a LogResponse object.
 * The function generates a custom log id, if not provided, by concatenating the blockHash, transactionHash, and logIndex values after removing the '0x' prefix.
 * It also converts string representations of blockNumber, transactionIndex, and logIndex to their corresponding numeric values.
 * Additionally, it creates EthAddress and TxHash instances for address and transactionHash fields, respectively.
 *
 * @param log - The RawLogResponse object to be converted.
 * @returns A LogResponse object with proper data types and structure.
 */
export function fromRawLogResponse(log: RawLogResponse): LogResponse {
  let id: string | null = log.id || null;

  // Generate a custom log id.
  if (
    typeof log.blockHash === 'string' &&
    typeof log.transactionHash === 'string' &&
    typeof log.logIndex === 'string'
  ) {
    const shaId = keccak256String(
      log.blockHash.replace('0x', '') + log.transactionHash.replace('0x', '') + log.logIndex.replace('0x', ''),
    );
    id = 'log_' + shaId.replace('0x', '').substring(0, 8);
  }

  const blockNumber = log.blockNumber !== null ? Number(log.blockNumber) : null;
  const transactionIndex = log.transactionIndex !== null ? Number(log.transactionIndex) : null;
  const logIndex = log.logIndex !== null ? Number(log.logIndex) : null;
  const address = EthAddress.fromString(log.address);
  const transactionHash = log.transactionHash !== null ? TxHash.fromString(log.transactionHash) : null;

  return { ...log, id, blockNumber, transactionIndex, transactionHash, logIndex, address };
}

/**
 * Converts a LogResponse object to its corresponding RawLogResponse format.
 * This function is used to revert the processed LogResponse back to its raw format,
 * primarily for compatibility with Ethereum JSON-RPC API or other libraries that
 * expect raw log data. The output will have all number properties converted to hex-strings
 * and addresses in lowercase representation where applicable.
 *
 * @param log - The LogResponse object to be converted to RawLogResponse format.
 * @returns A RawLogResponse object containing original raw log data.
 */
export function toRawLogResponse(log: LogResponse): RawLogResponse {
  const { id, blockNumber, transactionIndex, logIndex, address, transactionHash } = log;
  return {
    ...log,
    id: id ? id : undefined,
    blockNumber: blockNumber !== null ? numberToHex(blockNumber) : null,
    transactionIndex: transactionIndex !== null ? numberToHex(transactionIndex) : null,
    logIndex: logIndex !== null ? numberToHex(logIndex) : null,
    address: address.toString().toLowerCase(),
    transactionHash: transactionHash !== null ? transactionHash.toString() : null,
  };
}
