import { createDebugLogger } from '@aztec/foundation/log';
import { Tx, TxHash } from '@aztec/types';

import { TxPool } from './index.js';

/**
 * In-memory implementation of the Transaction Pool.
 */
export class InMemoryTxPool implements TxPool {
  /**
   * Our tx pool, stored as a Map in-memory, with K: tx hash and V: the transaction.
   */
  private txs: Map<bigint, Tx>;

  /**
   * Class constructor for in-memory TxPool. Initiates our transaction pool as a JS Map.
   * @param log - A logger.
   */
  constructor(private log = createDebugLogger('aztec:tx_pool')) {
    this.txs = new Map<bigint, Tx>();
  }

  /**
   * Checks if a transaction exists in the pool and returns it.
   * @param txHash - The generated tx hash.
   * @returns The transaction, if found, 'undefined' otherwise.
   */
  public getTxByHash(txHash: TxHash): Tx | undefined {
    const result = this.txs.get(txHash.toBigInt());
    return result;
  }

  /**
   * Adds a list of transactions to the pool. Duplicates are ignored.
   * @param txs - An array of txs to be added to the pool.
   */
  public async addTxs(txs: Tx[]): Promise<void> {
    for (const tx of txs) {
      this.log(`Adding tx with id ${await tx.getTxHash().then(t => t.toString())}`);
      this.txs.set(await tx.getTxHash().then(t => t.toBigInt()), tx);
    }
  }

  /**
   * Deletes transactions from the pool. Tx hashes that are not present are ignored.
   * @param txHashes - An array of tx hashes to be removed from the tx pool.
   * @returns The number of transactions that was deleted from the pool.
   */
  public deleteTxs(txHashes: TxHash[]): number {
    const numTxsRemoved = txHashes
      .map(txHash => this.txs.delete(txHash.toBigInt()))
      .filter(result => result === true).length;
    return numTxsRemoved;
  }

  /**
   * Gets all the transactions stored in the pool.
   * @returns Array of tx objects in the order they were added to the pool.
   */
  public getAllTxs(): Tx[] {
    return Array.from(this.txs.values());
  }
}
