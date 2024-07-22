import { createDebugLogger } from '@aztec/foundation/log';

import { type Database, type Key, type RootDatabase, open } from 'lmdb';

import { type AztecArray } from '../interfaces/array.js';
import { type AztecCounter } from '../interfaces/counter.js';
import { type AztecMap, type AztecMultiMap } from '../interfaces/map.js';
import { type AztecSet } from '../interfaces/set.js';
import { type AztecSingleton } from '../interfaces/singleton.js';
import { type AztecKVStore } from '../interfaces/store.js';
import { LmdbAztecArray } from './array.js';
import { LmdbAztecCounter } from './counter.js';
import { LmdbAztecMap } from './map.js';
import { LmdbAztecSet } from './set.js';
import { LmdbAztecSingleton } from './singleton.js';

/**
 * A key-value store backed by LMDB.
 */
export class AztecLmdbStore implements AztecKVStore {
  #rootDb: RootDatabase;
  #data: Database<unknown, Key>;
  #multiMapData: Database<unknown, Key>;

  constructor(rootDb: RootDatabase) {
    this.#rootDb = rootDb;

    // big bucket to store all the data
    this.#data = rootDb.openDB('data', {
      encoding: 'msgpack',
      keyEncoding: 'ordered-binary',
    });

    this.#multiMapData = rootDb.openDB('data_dup_sort', {
      encoding: 'ordered-binary',
      keyEncoding: 'ordered-binary',
      dupSort: true,
    });
  }

  /**
   * Creates a new AztecKVStore backed by LMDB. The path to the database is optional. If not provided,
   * the database will be stored in a temporary location and be deleted when the process exists.
   *
   * The `rollupAddress` passed is checked against what is stored in the database. If they do not match,
   * the database is cleared before returning the store. This way data is not accidentally shared between
   * different rollup instances.
   *
   * @param path - A path on the disk to store the database. Optional
   * @param ephemeral - true if the store should only exist in memory and not automatically be flushed to disk. Optional
   * @param log - A logger to use. Optional
   * @returns The store
   */
  static open(
    path?: string,
    ephemeral: boolean = false,
    log = createDebugLogger('aztec:kv-store:lmdb'),
  ): AztecLmdbStore {
    log.info(`Opening LMDB database at ${path || 'temporary location'}`);
    const rootDb = open({
      path,
      noSync: ephemeral,
    });
    return new AztecLmdbStore(rootDb);
  }

  /**
   * Creates a new AztecMap in the store.
   * @param name - Name of the map
   * @returns A new AztecMap
   */
  openMap<K extends string | number, V>(name: string): AztecMap<K, V> {
    return new LmdbAztecMap(this.#data, name);
  }

  /**
   * Creates a new AztecSet in the store.
   * @param name - Name of the set
   * @returns A new AztecSet
   */
  openSet<K extends string | number>(name: string): AztecSet<K> {
    return new LmdbAztecSet(this.#data, name);
  }

  /**
   * Creates a new AztecMultiMap in the store. A multi-map stores multiple values for a single key automatically.
   * @param name - Name of the map
   * @returns A new AztecMultiMap
   */
  openMultiMap<K extends string | number, V>(name: string): AztecMultiMap<K, V> {
    return new LmdbAztecMap(this.#multiMapData, name);
  }

  openCounter<K extends string | number | Array<string | number>>(name: string): AztecCounter<K> {
    return new LmdbAztecCounter(this.#data, name);
  }

  /**
   * Creates a new AztecArray in the store.
   * @param name - Name of the array
   * @returns A new AztecArray
   */
  openArray<T>(name: string): AztecArray<T> {
    return new LmdbAztecArray(this.#data, name);
  }

  /**
   * Creates a new AztecSingleton in the store.
   * @param name - Name of the singleton
   * @returns A new AztecSingleton
   */
  openSingleton<T>(name: string): AztecSingleton<T> {
    return new LmdbAztecSingleton(this.#data, name);
  }

  /**
   * Runs a callback in a transaction.
   * @param callback - Function to execute in a transaction
   * @returns A promise that resolves to the return value of the callback
   */
  transaction<T>(callback: () => T): Promise<T> {
    return this.#rootDb.transaction(callback);
  }

  /**
   * Clears the store
   */
  async clear() {
    await this.#rootDb.clearAsync();
  }
}
