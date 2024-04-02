import { Tx, TxHash } from '@aztec/circuit-types';
import { type TxAddedToPoolStats } from '@aztec/circuit-types/stats';
import { createDebugLogger } from '@aztec/foundation/log';

import { type TxPool } from './tx_pool.js';

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
    return result === undefined ? undefined : Tx.clone(result);
  }

  /**
   * Adds a list of transactions to the pool. Duplicates are ignored.
   * @param txs - An array of txs to be added to the pool.
   * @returns Empty promise.
   */
  public addTxs(txs: Tx[]): Promise<void> {
    for (const tx of txs) {
      const txHash = tx.getTxHash();
      this.log(`Adding tx with id ${txHash.toString()}`, {
        eventName: 'tx-added-to-pool',
        ...tx.getStats(),
      } satisfies TxAddedToPoolStats);
      this.txs.set(txHash.toBigInt(), tx);
    }
    return Promise.resolve();
  }

  /**
   * Deletes transactions from the pool. Tx hashes that are not present are ignored.
   * @param txHashes - An array of tx hashes to be removed from the tx pool.
   * @returns The number of transactions that was deleted from the pool.
   */
  public deleteTxs(txHashes: TxHash[]): Promise<void> {
    for (const txHash of txHashes) {
      this.txs.delete(txHash.toBigInt());
    }
    return Promise.resolve();
  }

  /**
   * Gets all the transactions stored in the pool.
   * @returns Array of tx objects in the order they were added to the pool.
   */
  public getAllTxs(): Tx[] {
    return Array.from(this.txs.values()).map(x => Tx.clone(x));
  }

  /**
   * Gets the hashes of all transactions currently in the tx pool.
   * @returns An array of transaction hashes found in the tx pool.
   */
  public getAllTxHashes(): TxHash[] {
    return Array.from(this.txs.keys()).map(x => TxHash.fromBigInt(x));
  }

  /**
   * Returns a boolean indicating if the transaction is present in the pool.
   * @param txHash - The hash of the transaction to be queried.
   * @returns True if the transaction present, false otherwise.
   */
  public hasTx(txHash: TxHash): boolean {
    return this.txs.has(txHash.toBigInt());
  }
}
