import { type Tx, type TxHash } from '@aztec/circuit-types';

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
   * Marks the set of txs as mined, as opposed to pending.
   * @param txHashes - Hashes of the txs to flag as mined.
   */
  markAsMined(txHashes: TxHash[]): Promise<void>;

  /**
   * Deletes transactions from the pool. Tx hashes that are not present are ignored.
   * @param txHashes - An array of tx hashes to be removed from the tx pool.
   */
  deleteTxs(txHashes: TxHash[]): Promise<void>;

  /**
   * Gets all transactions currently in the tx pool.
   * @returns An array of transaction objects found in the tx pool.
   */
  getAllTxs(): Tx[];

  /**
   * Gets the hashes of all transactions currently in the tx pool.
   * @returns An array of transaction hashes found in the tx pool.
   */
  getAllTxHashes(): TxHash[];

  /**
   * Gets the hashes of pending transactions currently in the tx pool.
   * @returns An array of pending transaction hashes found in the tx pool.
   */
  getPendingTxHashes(): TxHash[];

  /**
   * Gets the hashes of mined transactions currently in the tx pool.
   * @returns An array of mined transaction hashes found in the tx pool.
   */
  getMinedTxHashes(): TxHash[];

  /**
   * Returns whether the given tx hash is flagged as pending or mined.
   * @param txHash - Hash of the tx to query.
   * @returns Pending or mined depending on its status, or undefined if not found.
   */
  getTxStatus(txHash: TxHash): 'pending' | 'mined' | undefined;
}
