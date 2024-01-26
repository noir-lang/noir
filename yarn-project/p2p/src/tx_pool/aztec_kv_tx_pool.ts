import { Tx, TxHash } from '@aztec/circuit-types';
import { TxAddedToPoolStats } from '@aztec/circuit-types/stats';
import { Logger, createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore, AztecMap } from '@aztec/kv-store';

import { TxPool } from './tx_pool.js';

/**
 * In-memory implementation of the Transaction Pool.
 */
export class AztecKVTxPool implements TxPool {
  #store: AztecKVStore;

  /**
   * Our tx pool, stored as a Map in-memory, with K: tx hash and V: the transaction.
   */
  #txs: AztecMap<string, Buffer>;

  #log: Logger;

  /**
   * Class constructor for in-memory TxPool. Initiates our transaction pool as a JS Map.
   * @param store - A KV store.
   * @param log - A logger.
   */
  constructor(store: AztecKVStore, log = createDebugLogger('aztec:tx_pool')) {
    this.#txs = store.openMap('txs');
    this.#store = store;
    this.#log = log;
  }

  /**
   * Checks if a transaction exists in the pool and returns it.
   * @param txHash - The generated tx hash.
   * @returns The transaction, if found, 'undefined' otherwise.
   */
  public getTxByHash(txHash: TxHash): Tx | undefined {
    const buffer = this.#txs.get(txHash.toString());
    return buffer ? Tx.fromBuffer(buffer) : undefined;
  }

  /**
   * Adds a list of transactions to the pool. Duplicates are ignored.
   * @param txs - An array of txs to be added to the pool.
   * @returns Empty promise.
   */
  public async addTxs(txs: Tx[]): Promise<void> {
    const txHashes = await Promise.all(txs.map(tx => tx.getTxHash()));
    return this.#store.transaction(() => {
      for (const [i, tx] of txs.entries()) {
        const txHash = txHashes[i];
        this.#log.info(`Adding tx with id ${txHash.toString()}`, {
          eventName: 'tx-added-to-pool',
          ...tx.getStats(),
        } satisfies TxAddedToPoolStats);

        void this.#txs.set(txHash.toString(), tx.toBuffer());
      }
    });
  }

  /**
   * Deletes transactions from the pool. Tx hashes that are not present are ignored.
   * @param txHashes - An array of tx hashes to be removed from the tx pool.
   * @returns The number of transactions that was deleted from the pool.
   */
  public deleteTxs(txHashes: TxHash[]): Promise<void> {
    return this.#store.transaction(() => {
      for (const hash of txHashes) {
        void this.#txs.delete(hash.toString());
      }
    });
  }

  /**
   * Gets all the transactions stored in the pool.
   * @returns Array of tx objects in the order they were added to the pool.
   */
  public getAllTxs(): Tx[] {
    return Array.from(this.#txs.values()).map(buffer => Tx.fromBuffer(buffer));
  }

  /**
   * Gets the hashes of all transactions currently in the tx pool.
   * @returns An array of transaction hashes found in the tx pool.
   */
  public getAllTxHashes(): TxHash[] {
    return Array.from(this.#txs.keys()).map(x => TxHash.fromString(x));
  }

  /**
   * Returns a boolean indicating if the transaction is present in the pool.
   * @param txHash - The hash of the transaction to be queried.
   * @returns True if the transaction present, false otherwise.
   */
  public hasTx(txHash: TxHash): boolean {
    return this.#txs.has(txHash.toString());
  }
}
