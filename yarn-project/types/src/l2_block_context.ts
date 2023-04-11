import { KERNEL_NEW_COMMITMENTS_LENGTH } from '@aztec/circuits.js';
import { keccak } from '@aztec/foundation';
import { L2Block } from './l2_block.js';
import { TxHash } from './tx_hash.js';

export class L2BlockContext {
  private txHashes: (TxHash | undefined)[];
  private blockHash: Buffer | undefined;

  constructor(public readonly block: L2Block) {
    this.txHashes = new Array(Math.floor(block.newCommitments.length / KERNEL_NEW_COMMITMENTS_LENGTH));
  }

  public getBlockHash(): Buffer {
    if (!this.blockHash) {
      this.blockHash = keccak(this.block.encode());
    }
    return this.blockHash;
  }

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
