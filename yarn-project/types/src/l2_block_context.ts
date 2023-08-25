import { L2Block } from './l2_block.js';
import { TxHash } from './tx/tx_hash.js';

/**
 * A wrapper around L2 block used to cache results of expensive operations.
 */
export class L2BlockContext {
  private txHashes: TxHash[] | undefined;

  constructor(
    /**
     * The underlying L2 block.
     */
    public readonly block: L2Block,
  ) {}

  /**
   * Returns the tx hash of the tx in the block at the given index.
   * @param txIndex - The index of the tx.
   * @returns The tx's hash.
   */
  public getTxHash(txIndex: number): TxHash {
    return this.txHashes ? this.txHashes[txIndex] : this.block.getTx(txIndex).txHash;
  }

  /**
   * Returns the tx hashes of all txs in the block.
   * @returns The tx hashes.
   */
  public getTxHashes(): TxHash[] {
    // First ensure that all tx hashes are calculated
    if (!this.txHashes) {
      this.txHashes = this.block.getTxs().map(tx => tx.txHash);
    }
    return this.txHashes;
  }
}
