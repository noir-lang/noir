import { type AztecArray } from './array.js';
import { type Key } from './common.js';
import { type AztecCounter } from './counter.js';
import { type AztecMap, type AztecMultiMap } from './map.js';
import { type AztecSet } from './set.js';
import { type AztecSingleton } from './singleton.js';

/** A key-value store */
export interface AztecKVStore {
  /**
   * Creates a new map.
   * @param name - The name of the map
   * @returns The map
   */
  openMap<K extends string | number, V>(name: string): AztecMap<K, V>;

  /**
   * Creates a new set.
   * @param name - The name of the set
   * @returns The set
   */
  openSet<K extends string | number>(name: string): AztecSet<K>;

  /**
   * Creates a new multi-map.
   * @param name - The name of the multi-map
   * @returns The multi-map
   */
  openMultiMap<K extends string | number, V>(name: string): AztecMultiMap<K, V>;

  /**
   * Creates a new array.
   * @param name - The name of the array
   * @returns The array
   */
  openArray<T>(name: string): AztecArray<T>;

  /**
   * Creates a new singleton.
   * @param name - The name of the singleton
   * @returns The singleton
   */
  openSingleton<T>(name: string): AztecSingleton<T>;

  /**
   * Creates a new count map.
   * @param name - name of the counter
   */
  openCounter<K extends Key>(name: string): AztecCounter<K>;

  /**
   * Starts a transaction. All calls to read/write data while in a transaction are queued and executed atomically.
   * @param callback - The callback to execute in a transaction
   */
  transaction<T extends Exclude<any, Promise<any>>>(callback: () => T): Promise<T>;

  /**
   * Clears the store
   */
  clear(): Promise<void>;
}
