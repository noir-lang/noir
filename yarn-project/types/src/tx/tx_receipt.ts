import { AztecAddress } from '@aztec/foundation/aztec-address';
import { TxHash } from '@aztec/types';

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
     * The hash of the block containing the transaction.
     */
    public blockHash?: Buffer,
    /**
     * The block number in which the transaction was included.
     */
    public blockNumber?: number,
    /**
     * The sender's address.
     */
    public origin?: AztecAddress,
    /**
     * The deployed contract's address.
     */
    public contractAddress?: AztecAddress,
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
      origin: this.origin?.toString(),
      contractAddress: this.contractAddress?.toString(),
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
    const blockHash = obj.blockHash ? Buffer.from(obj.blockHash, 'hex') : undefined;
    const blockNumber = obj.blockNumber ? Number(obj.blockNumber) : undefined;
    const origin = obj.origin ? AztecAddress.fromString(obj.origin) : undefined;
    const contractAddress = obj.contractAddress ? AztecAddress.fromString(obj.contractAddress) : undefined;
    return new TxReceipt(txHash, status, error, blockHash, blockNumber, origin, contractAddress);
  }
}
