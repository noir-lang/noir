import { Tx, TxHash } from '@aztec/circuit-types';
import { type TxAddedToPoolStats } from '@aztec/circuit-types/stats';
import { type Logger, createDebugLogger } from '@aztec/foundation/log';
import { type AztecKVStore, type AztecMap, type AztecSet } from '@aztec/kv-store';
import { type TelemetryClient } from '@aztec/telemetry-client';

import { TxPoolInstrumentation } from './instrumentation.js';
import { type TxPool } from './tx_pool.js';

/**
 * In-memory implementation of the Transaction Pool.
 */
export class AztecKVTxPool implements TxPool {
  #store: AztecKVStore;

  /** Our tx pool, stored as a Map, with K: tx hash and V: the transaction. */
  #txs: AztecMap<string, Buffer>;

  /** Index for pending txs. */
  #pendingTxs: AztecSet<string>;
  /** Index for mined txs. */
  #minedTxs: AztecSet<string>;

  #log: Logger;

  #metrics: TxPoolInstrumentation;

  /**
   * Class constructor for in-memory TxPool. Initiates our transaction pool as a JS Map.
   * @param store - A KV store.
   * @param log - A logger.
   */
  constructor(store: AztecKVStore, telemetry: TelemetryClient, log = createDebugLogger('aztec:tx_pool')) {
    this.#txs = store.openMap('txs');
    this.#minedTxs = store.openSet('minedTxs');
    this.#pendingTxs = store.openSet('pendingTxs');

    this.#store = store;
    this.#log = log;
    this.#metrics = new TxPoolInstrumentation(telemetry, 'AztecKVTxPool');
  }

  public markAsMined(txHashes: TxHash[]): Promise<void> {
    return this.#store.transaction(() => {
      for (const hash of txHashes) {
        const key = hash.toString();
        void this.#minedTxs.add(key);
        void this.#pendingTxs.delete(key);
      }
    });
  }

  public getPendingTxHashes(): TxHash[] {
    return Array.from(this.#pendingTxs.entries()).map(x => TxHash.fromString(x));
  }

  public getMinedTxHashes(): TxHash[] {
    return Array.from(this.#minedTxs.entries()).map(x => TxHash.fromString(x));
  }

  public getTxStatus(txHash: TxHash): 'pending' | 'mined' | undefined {
    const key = txHash.toString();
    if (this.#pendingTxs.has(key)) {
      return 'pending';
    } else if (this.#minedTxs.has(key)) {
      return 'mined';
    } else {
      return undefined;
    }
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
  public addTxs(txs: Tx[]): Promise<void> {
    const txHashes = txs.map(tx => tx.getTxHash());
    return this.#store.transaction(() => {
      for (const [i, tx] of txs.entries()) {
        const txHash = txHashes[i];
        this.#log.info(`Adding tx with id ${txHash.toString()}`, {
          eventName: 'tx-added-to-pool',
          ...tx.getStats(),
        } satisfies TxAddedToPoolStats);

        const key = txHash.toString();
        void this.#txs.set(key, tx.toBuffer());
        if (!this.#minedTxs.has(key)) {
          // REFACTOR: Use an lmdb conditional write to avoid race conditions with this write tx
          void this.#pendingTxs.add(key);
        }
      }

      this.#metrics.recordTxs(txs);
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
        const key = hash.toString();
        void this.#txs.delete(key);
        void this.#pendingTxs.delete(key);
        void this.#minedTxs.delete(key);
      }

      this.#metrics.removeTxs(txHashes.length);
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
}
