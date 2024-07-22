import { type Key, type Range } from './common.js';

/**
 * A set backed by a persistent store.
 */
export interface AztecSet<K extends Key> {
  /**
   * Checks if a key exists in the set.
   * @param key - The key to check
   * @returns True if the key exists, false otherwise
   */
  has(key: K): boolean;

  /**
   * Adds the given value.
   * @param key - The key to add.
   */
  add(key: K): Promise<void>;

  /**
   * Deletes the given key.
   * @param key - The key to delete.
   */
  delete(key: K): Promise<void>;

  /**
   * Iterates over the sets's keys entries in the key's natural order
   * @param range - The range of keys to iterate over
   */
  entries(range?: Range<K>): IterableIterator<K>;
}
