import { MAX_NEW_COMMITMENTS_PER_TX } from '@aztec/circuits.js';
import { keccak } from '@aztec/foundation/crypto';

import { L2Block } from './l2_block.js';
import { TxHash } from './tx/tx_hash.js';

/**
 * A wrapper around L2 block used to cache results of expensive operations.
 */
export class L2BlockContext {
  private txHashes: (TxHash | undefined)[];
  private blockHash: Buffer | undefined;

  constructor(
    /**
     * The underlying L2 block.
     */
    public readonly block: L2Block,
  ) {
    this.txHashes = new Array(Math.floor(block.newCommitments.length / MAX_NEW_COMMITMENTS_PER_TX));
  }

  /**
   * Returns the underlying block's hash.
   * @returns The block's hash.
   */
  public getBlockHash(): Buffer {
    if (!this.blockHash) {
      this.blockHash = keccak(this.block.encode());
    }
    return this.blockHash;
  }

  /**
   * Returns the tx hash of the tx in the block at the given index.
   * @param txIndex - The index of the tx.
   * @returns The tx's hash.
   */
  public getTxHash(txIndex: number): TxHash {
    if (!this.txHashes[txIndex]) {
      const txHash = this.block.getTx(txIndex).txHash;
      if (txHash === undefined) {
        throw new Error(`Tx hash for tx ${txIndex} in block ${this.block.number} is undefined`);
      }
      this.txHashes[txIndex] = txHash;
    }
    return this.txHashes[txIndex]!;
  }

  /**
   * Returns the tx hashes of all txs in the block.
   * @returns The tx hashes.
   */
  public getTxHashes(): TxHash[] {
    // First ensure that all tx hashes are calculated
    for (let txIndex = 0; txIndex < this.txHashes.length; txIndex++) {
      if (!this.txHashes[txIndex]) {
        this.txHashes[txIndex] = this.block.getTx(txIndex).txHash;
      }
    }
    return this.txHashes as TxHash[];
  }
}
