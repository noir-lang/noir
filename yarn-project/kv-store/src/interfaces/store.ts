import { AztecArray } from './array.js';
import { Key } from './common.js';
import { AztecCounter } from './counter.js';
import { AztecMap, AztecMultiMap } from './map.js';
import { AztecSingleton } from './singleton.js';

/** A key-value store */
export interface AztecKVStore {
  /**
   * Creates a new map.
   * @param name - The name of the map
   * @returns The map
   */
  createMap<K extends string | number, V>(name: string): AztecMap<K, V>;

  /**
   * Creates a new multi-map.
   * @param name - The name of the multi-map
   * @returns The multi-map
   */
  createMultiMap<K extends string | number, V>(name: string): AztecMultiMap<K, V>;

  /**
   * Creates a new array.
   * @param name - The name of the array
   * @returns The array
   */
  createArray<T>(name: string): AztecArray<T>;

  /**
   * Creates a new singleton.
   * @param name - The name of the singleton
   * @returns The singleton
   */
  createSingleton<T>(name: string): AztecSingleton<T>;

  /**
   * Creates a new count map.
   * @param name - name of the counter
   */
  createCounter<K extends Key>(name: string): AztecCounter<K>;

  /**
   * Starts a transaction. All calls to read/write data while in a transaction are queued and executed atomically.
   * @param callback - The callback to execute in a transaction
   */
  transaction<T extends Exclude<any, Promise<any>>>(callback: () => T): Promise<T>;
}
