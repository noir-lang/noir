import { Tx, TxHash } from '@aztec/types';

/**
 * Interface of a transaction pool. The pool includes tx requests and is kept up-to-date by a P2P client.
 */
export interface TxPool {
  /**
   * Adds a list of transactions to the pool. Duplicates are ignored.
   * @param txs - An array of txs to be added to the pool.
   */
  addTxs(txs: Tx[]): Promise<void>;

  /**
   * Checks if a transaction exists in the pool and returns it.
   * @param txHash - The hash of the transaction, used as an ID.
   * @returns The transaction, if found, 'undefined' otherwise.
   */
  getTxByHash(txHash: TxHash): Tx | undefined;

  /**
   * Deletes transactions from the pool. Tx hasehs that are not present are ignored.
   * @param txHashes - An array of tx hashes to be removed from the tx pool.
   */
  deleteTxs(txHashes: TxHash[]): void;

  /**
   * Gets all transactions currently in the tx pool.
   * @returns An array of transaction objects found in the tx pool.
   */
  getAllTxs(): Tx[];
}
