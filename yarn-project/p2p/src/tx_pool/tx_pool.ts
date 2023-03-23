import { Tx } from '../index.js';

/**
 * Interface of a transaction pool. The pool includes tx requests and is kept up-to-date by a P2P client.
 */
export interface TxPool {
  /**
   * Adds a list of transactions to the pool. Duplicates are ignored.
   * @param txs - An array of txs to be added to the pool.
   */
  addTxs(txs: Tx[]): void;

  /**
   * Checks if a transaction exists in the pool and returns it.
   * @param txId - The generated tx ID.
   * @returns The transaction, if found, 'undefined' otherwise.
   */
  getTx(txId: Buffer): Tx | undefined;

  /**
   * Deletes transactions from the pool. Tx IDs that are not present are ignored.
   * @param txIds - An array of tx IDs to be removed from the tx pool.
   */
  deleteTxs(txIds: Buffer[]): void;

  /**
   * Gets all transactions currently in the tx pool.
   * @returns An array of transaction objects found in the tx pool.
   */
  getAllTxs(): Tx[];
}
