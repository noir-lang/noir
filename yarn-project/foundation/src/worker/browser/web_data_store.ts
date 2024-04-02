import levelup, { type LevelUp } from 'levelup';
import memdown from 'memdown';

import { type DataStore } from '../data_store.js';

/**
 * Cache for data used by wasm module.
 * Stores in a LevelUp database.
 */
export class WebDataStore implements DataStore {
  private db: LevelUp;

  constructor() {
    // TODO: The whole point of this is to reduce memory load in the browser.
    // Replace with leveljs so the data is stored in indexeddb and not in memory.
    // Hack: Cast as any to work around package "broken" with node16 resolution
    // See https://github.com/microsoft/TypeScript/issues/49160
    this.db = levelup((memdown as any)());
  }

  /**
   * Lookup a key.
   * @param key - Key to lookup.
   * @returns The buffer.
   */
  async get(key: string): Promise<Buffer | undefined> {
    return await this.db.get(key).catch(() => {});
  }

  /**
   * Alter a key.
   * @param key - Key to alter.
   * @param value - Buffer to store.
   */
  async set(key: string, value: Buffer): Promise<void> {
    await this.db.put(key, value);
  }
}
