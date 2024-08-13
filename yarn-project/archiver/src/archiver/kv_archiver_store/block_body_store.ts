import { Body } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { type AztecKVStore, type AztecMap, type AztecSingleton } from '@aztec/kv-store';

import { type DataRetrieval } from '../data_retrieval.js';

export class BlockBodyStore {
  /** Map block body hash to block body */
  #blockBodies: AztecMap<string, Buffer>;

  /** Stores L1 block number in which the last processed L2 block body was included */
  #lastSynchedL1Block: AztecSingleton<bigint>;

  constructor(private db: AztecKVStore, private log = createDebugLogger('aztec:archiver:block_body_store')) {
    this.#blockBodies = db.openMap('archiver_block_bodies');
    this.#lastSynchedL1Block = db.openSingleton('archiver_block_bodies_last_synched_l1_block');
  }

  /**
   * Append new block bodies to the store's map.
   * @param blockBodies - The L2 block bodies to be added to the store.
   * @returns True if the operation is successful.
   */
  addBlockBodies(blockBodies: DataRetrieval<Body>): Promise<boolean> {
    return this.db.transaction(() => {
      for (const body of blockBodies.retrievedData) {
        void this.#blockBodies.set(body.getTxsEffectsHash().toString('hex'), body.toBuffer());
      }
      void this.#lastSynchedL1Block.set(blockBodies.lastProcessedL1BlockNumber);
      return true;
    });
  }

  /**
   * Gets a list of L2 block bodies with its associated txsEffectsHashes
   * @param txsEffectsHashes - The txsEffectsHashes list that corresponds to the blockBodies we want to retrieve
   * @returns The requested L2 block bodies
   */
  async getBlockBodies(txsEffectsHashes: Buffer[]): Promise<(Body | undefined)[]> {
    const blockBodiesBuffer = await this.db.transaction(() =>
      txsEffectsHashes.map(txsEffectsHash => this.#blockBodies.get(txsEffectsHash.toString('hex'))),
    );

    const blockBodies: (Body | undefined)[] = [];
    for (let i = 0; i < blockBodiesBuffer.length; i++) {
      const blockBodyBuffer = blockBodiesBuffer[i];
      if (blockBodyBuffer === undefined) {
        this.log.warn(`Block body buffer is undefined for txsEffectsHash: ${txsEffectsHashes[i].toString('hex')}`);
      }
      blockBodies.push(blockBodyBuffer ? Body.fromBuffer(blockBodyBuffer) : undefined);
    }

    return blockBodies;
  }

  /**
   * Gets an L2 block body.
   * @param txsEffectsHash - The txHash of the block body to return
   * @returns The requested L2 block body
   */
  getBlockBody(txsEffectsHash: Buffer): Body | undefined {
    const blockBody = this.#blockBodies.get(txsEffectsHash.toString('hex'));

    return blockBody && Body.fromBuffer(blockBody);
  }

  /**
   * Gets the last L1 block number in which a L2 block body was included
   * @returns The L1 block number
   */
  getSynchedL1BlockNumber(): bigint {
    return this.#lastSynchedL1Block.get() ?? 0n;
  }
}
