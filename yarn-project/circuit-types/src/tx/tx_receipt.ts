import { RevertCode } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

import { type UniqueNote } from '../notes/extended_note.js';
import { type PublicDataWrite } from '../public_data_write.js';
import { TxHash } from './tx_hash.js';

/**
 * Possible status of a transaction.
 */
export enum TxStatus {
  DROPPED = 'dropped',
  PENDING = 'pending',
  SUCCESS = 'success',
  APP_LOGIC_REVERTED = 'app_logic_reverted',
  TEARDOWN_REVERTED = 'teardown_reverted',
  BOTH_REVERTED = 'both_reverted',
}

/**
 * Represents a transaction receipt in the Aztec network.
 * Contains essential information about the transaction including its status, origin, and associated addresses.
 * REFACTOR: TxReceipt should be returned only once the tx is mined, and all its fields should be required.
 * We should not be using a TxReceipt to answer a query for a pending or dropped tx.
 */
export class TxReceipt {
  constructor(
    /**
     * A unique identifier for a transaction.
     */
    public txHash: TxHash,
    /**
     * The transaction's status.
     */
    public status: TxStatus,
    /**
     * Description of transaction error, if any.
     */
    public error: string,
    /**
     * The transaction fee paid for the transaction.
     */
    public transactionFee?: bigint,
    /**
     * The hash of the block containing the transaction.
     */
    public blockHash?: Buffer,
    /**
     * The block number in which the transaction was included.
     */
    public blockNumber?: number,
    /**
     * Information useful for testing/debugging, set when test flag is set to true in `waitOpts`.
     */
    public debugInfo?: DebugInfo,
  ) {}

  /**
   * Convert a Tx class object to a plain JSON object.
   * @returns A plain object with Tx properties.
   */
  public toJSON() {
    return {
      txHash: this.txHash.toString(),
      status: this.status.toString(),
      error: this.error,
      blockHash: this.blockHash?.toString('hex'),
      blockNumber: this.blockNumber,
      transactionFee: this.transactionFee?.toString(),
    };
  }

  /**
   * Convert a plain JSON object to a Tx class object.
   * @param obj - A plain Tx JSON object.
   * @returns A Tx class object.
   */
  public static fromJSON(obj: any) {
    const txHash = TxHash.fromString(obj.txHash);
    const status = obj.status as TxStatus;
    const error = obj.error;
    const transactionFee = obj.transactionFee ? BigInt(obj.transactionFee) : undefined;
    const blockHash = obj.blockHash ? Buffer.from(obj.blockHash, 'hex') : undefined;
    const blockNumber = obj.blockNumber ? Number(obj.blockNumber) : undefined;
    return new TxReceipt(txHash, status, error, transactionFee, blockHash, blockNumber);
  }

  public static statusFromRevertCode(revertCode: RevertCode) {
    if (revertCode.equals(RevertCode.OK)) {
      return TxStatus.SUCCESS;
    } else if (revertCode.equals(RevertCode.APP_LOGIC_REVERTED)) {
      return TxStatus.APP_LOGIC_REVERTED;
    } else if (revertCode.equals(RevertCode.TEARDOWN_REVERTED)) {
      return TxStatus.TEARDOWN_REVERTED;
    } else if (revertCode.equals(RevertCode.BOTH_REVERTED)) {
      return TxStatus.BOTH_REVERTED;
    } else {
      throw new Error(`Unknown revert code: ${revertCode}`);
    }
  }
}

/**
 * Information useful for debugging/testing purposes included in the receipt when the debug flag is set to true
 * in `WaitOpts`.
 */
interface DebugInfo {
  /**
   * New note hashes created by the transaction.
   */
  noteHashes: Fr[];
  /**
   * New nullifiers created by the transaction.
   */
  nullifiers: Fr[];
  /**
   * New public data writes created by the transaction.
   */
  publicDataWrites: PublicDataWrite[];
  /**
   * New L2 to L1 messages created by the transaction.
   */
  l2ToL1Msgs: Fr[];
  /**
   * Notes created in this tx which were successfully decoded with the incoming keys of accounts which are registered
   * in the PXE which was used to submit the tx. You will not get notes of accounts which are not registered in
   * the PXE here even though they were created in this tx.
   */
  visibleIncomingNotes: UniqueNote[];
  /**
   * Notes created in this tx which were successfully decoded with the outgoing keys of accounts which are registered
   * in the PXE which was used to submit the tx. You will not get notes of accounts which are not registered in
   * the PXE here even though they were created in this tx.
   */
  visibleOutgoingNotes: UniqueNote[];
}
