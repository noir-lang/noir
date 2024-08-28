import { type AztecKVStore, type AztecSingleton } from '@aztec/kv-store';

import { type SingletonDataRetrieval } from '../structs/data_retrieval.js';

export class ProvenStore {
  /** Stores L1 block number in which the last processed L2 block was included */
  #lastSynchedL1Block: AztecSingleton<bigint>;

  /** Stores last proven L2 block number */
  #lastProvenL2Block: AztecSingleton<number>;

  constructor(private db: AztecKVStore) {
    this.#lastSynchedL1Block = db.openSingleton('archiver_last_l1_block_proven_logs');
    this.#lastProvenL2Block = db.openSingleton('archiver_last_proven_l2_block');
  }

  /**
   * Gets the most recent L1 block processed.
   */
  getSynchedL1BlockNumber(): bigint {
    return this.#lastSynchedL1Block.get() ?? 0n;
  }

  getProvenL2BlockNumber(): number {
    return this.#lastProvenL2Block.get() ?? 0;
  }

  async setProvenL2BlockNumber(blockNumber: SingletonDataRetrieval<number>) {
    await this.db.transaction(() => {
      void this.#lastProvenL2Block.set(blockNumber.retrievedData);
      void this.#lastSynchedL1Block.set(blockNumber.lastProcessedL1BlockNumber);
    });
  }
}
