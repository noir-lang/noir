import { TxHash } from '@aztec/types';
import { AztecAddress } from '@aztec/foundation/aztec-address';

/**
 * Possible status of a transaction.
 */
export enum TxStatus {
  DROPPED = 'dropped',
  MINED = 'mined',
  PENDING = 'pending',
}

/**
 * Represents a transaction receipt in the Aztec network.
 * Contains essential information about the transaction including its status, origin, and associated addresses.
 */
export interface TxReceipt {
  /**
   * A unique identifier for a transaction.
   */
  txHash: TxHash;
  /**
   * The hash of the block containing the transaction.
   */
  blockHash?: Buffer;
  /**
   * The block number in which the transaction was included.
   */
  blockNumber?: number;
  /**
   * The sender's address.
   */
  origin?: AztecAddress;
  /**
   * The deployed contract's address.
   */
  contractAddress?: AztecAddress;
  /**
   * The transaction's status.
   */
  status: TxStatus;
  /**
   * Description of transaction error, if any.
   */
  error: string;
}
