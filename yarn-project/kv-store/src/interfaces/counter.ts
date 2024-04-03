import { type Key, type Range } from './common.js';

/**
 * A map that counts how many times it sees a key. Once 0 is reached,  that key is removed from the map.
 * Iterating over the map will only return keys that have a count over 0.
 *
 * Keys are stored in sorted order
 */
export interface AztecCounter<K extends Key = Key> {
  /**
   * Resets the count of the given key to the given value.
   * @param key - The key to reset
   * @param value - The value to reset the key to
   */
  set(key: K, value: number): Promise<void>;

  /**
   * Updates the count of the given key by the given delta. This can be used to increment or decrement the count.
   * Once a key's count reaches 0, it is removed from the map.
   *
   * @param key - The key to update
   * @param delta - The amount to modify the key by
   */
  update(key: K, delta: number): Promise<void>;

  /**
   * Gets the current count.
   * @param key - The key to get the count of
   */
  get(key: K): number;

  /**
   * Returns keys in the map in sorted order. Only returns keys that have been seen at least once.
   * @param range - The range of keys to iterate over
   */
  keys(range: Range<K>): IterableIterator<K>;

  /**
   * Returns keys and their counts in the map sorted by the key. Only returns keys that have been seen at least once.
   * @param range - The range of keys to iterate over
   */
  entries(range: Range<K>): IterableIterator<[K, number]>;
}
