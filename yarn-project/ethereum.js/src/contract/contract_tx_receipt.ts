import { EthAddress } from '@aztec/foundation';
import { LogResponse, TransactionReceipt, TxHash } from '../eth_rpc/index.js';
import { DecodedError } from './decode_error.js';

/**
 * Represents a parsed Ethereum event log specific to Ethereum contracts.
 * Contains information about the event, such as its name, address, arguments, block data, and signature.
 * Useful for tracking contract interactions and state changes on the blockchain.
 */
export interface EventLog<Args, Name = string> {
  /**
   * A unique identifier for the event log.
   */
  id: string | null;
  /**
   * Indicates whether the event log has been removed due to a chain reorganization.
   */
  removed?: boolean;
  /**
   * The name of the emitted event.
   */
  event: Name;
  /**
   * The Ethereum address of the contract emitting the event.
   */
  address: EthAddress;
  /**
   * Arguments associated with the emitted event.
   */
  args: Args;
  /**
   * The index position of the log entry in the block.
   */
  logIndex: number | null;
  /**
   * The index of the transaction within the block containing it.
   */
  transactionIndex: number | null;
  /**
   * The unique identifier of the transaction.
   */
  transactionHash: TxHash | null;
  /**
   * The hash of the block containing this event.
   */
  blockHash: string | null;
  /**
   * The block number containing the event.
   */
  blockNumber: number | null;
  /**
   * Raw event data and topics emitted by the contract.
   */
  raw: {
    /**
     * The raw hexadecimal representation of the event data.
     */
    data: string;
    /**
     * An array of indexed event arguments encoded as hexadecimal strings.
     */
    topics: string[];
  };
  /**
   * The unique identifier of the event signature.
   */
  signature: string | null;
}

/**
 * Represents a contract transaction receipt in the Ethereum network.
 * Extends the standard transaction receipt with additional information about anonymous logs and
 * decoded events specific to the contract. It also includes optional error details in case of a failed transaction.
 */
export interface ContractTxReceipt<Events = void> extends TransactionReceipt {
  /**
   * An array of logs without specific event signatures.
   */
  anonymousLogs: LogResponse[];
  /**
   * An object containing arrays of various event logs, keyed by their respective event names.
   */
  events: Events extends void ? { [eventName: string]: EventLog<any>[] } : Events;
  /**
   * An optional field containing error information, including a message and decodedError if available.
   */
  error?: {
    /**
     * The human-readable error message.
     */
    message: string;
    /**
     * Decoded information from a failing transaction error.
     */
    decodedError?: DecodedError;
  };
}
