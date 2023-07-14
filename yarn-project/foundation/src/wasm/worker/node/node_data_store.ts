import leveldown from 'leveldown';
import levelup, { LevelUp } from 'levelup';
import memdown from 'memdown';

import { DataStore } from '../data_store.js';

/**
 * Cache for data used by wasm module.
 */
export class NodeDataStore implements DataStore {
  private db: LevelUp;

  // eslint-disable-next-line
  constructor(path?: string) {
    // Hack: Cast as any to work around packages "broken" with node16 resolution
    // See https://github.com/microsoft/TypeScript/issues/49160
    this.db = levelup(path ? (leveldown as any)(path) : (memdown as any)());
  }

  /**
   * Get a value from our DB.
   * @param key - The key to look up.
   * @returns The value.
   */
  async get(key: string): Promise<Buffer | undefined> {
    return await this.db.get(key).catch(() => {});
  }

  /**
   * Set a value in our DB.
   * @param key - The key to update.
   * @param value - The value to set.
   */
  async set(key: string, value: Buffer): Promise<void> {
    await this.db.put(key, value);
  }
}
