/**
 * A map backed by a persistent store.
 */
export interface AztecMap<K extends string | number, V> {
  /**
   * Gets the value at the given key.
   * @param key - The key to get the value from
   */
  get(key: K): V | undefined;

  /**
   * Checks if a key exists in the map.
   * @param key - The key to check
   * @returns True if the key exists, false otherwise
   */
  has(key: K): boolean;

  /**
   * Sets the value at the given key.
   * @param key - The key to set the value at
   * @param val - The value to set
   */
  set(key: K, val: V): Promise<boolean>;

  /**
   * Sets the value at the given key if it does not already exist.
   * @param key - The key to set the value at
   * @param val - The value to set
   */
  setIfNotExists(key: K, val: V): Promise<boolean>;

  /**
   * Deletes the value at the given key.
   * @param key - The key to delete the value at
   */
  delete(key: K): Promise<boolean>;

  /**
   * Iterates over the map's key-value entries
   */
  entries(): IterableIterator<[K, V]>;

  /**
   * Iterates over the map's values
   */
  values(): IterableIterator<V>;

  /**
   * Iterates over the map's keys
   */
  keys(): IterableIterator<K>;
}

/**
 * A map backed by a persistent store that can have multiple values for a single key.
 */
export interface AztecMultiMap<K extends string | number, V> extends AztecMap<K, V> {
  /**
   * Gets all the values at the given key.
   * @param key - The key to get the values from
   */
  getValues(key: K): IterableIterator<V>;

  /**
   * Deletes a specific value at the given key.
   * @param key - The key to delete the value at
   * @param val - The value to delete
   */
  deleteValue(key: K, val: V): Promise<void>;
}
