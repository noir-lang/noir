import { EthAddress } from '@aztec/foundation/eth-address';
import { Logger, createDebugLogger } from '@aztec/foundation/log';

import { Database, Key, RootDatabase, open } from 'lmdb';

import { AztecArray } from '../interfaces/array.js';
import { AztecCounter } from '../interfaces/counter.js';
import { AztecMap, AztecMultiMap } from '../interfaces/map.js';
import { AztecSingleton } from '../interfaces/singleton.js';
import { AztecKVStore } from '../interfaces/store.js';
import { LmdbAztecArray } from './array.js';
import { LmdbAztecCounter } from './counter.js';
import { LmdbAztecMap } from './map.js';
import { LmdbAztecSingleton } from './singleton.js';

/**
 * A key-value store backed by LMDB.
 */
export class AztecLmdbStore implements AztecKVStore {
  #rootDb: RootDatabase;
  #data: Database<unknown, Key>;
  #multiMapData: Database<unknown, Key>;
  #rollupAddress: AztecSingleton<string>;
  #log: Logger;

  constructor(rootDb: RootDatabase, log: Logger) {
    this.#rootDb = rootDb;
    this.#log = log;

    // big bucket to store all the data
    this.#data = rootDb.openDB('data', {
      encoding: 'msgpack',
      keyEncoding: 'ordered-binary',
    });

    this.#multiMapData = rootDb.openDB('data_dup_sort', {
      encoding: 'msgpack',
      keyEncoding: 'ordered-binary',
      dupSort: true,
    });

    this.#rollupAddress = this.openSingleton('rollupAddress');
  }

  /**
   * Creates a new AztecKVStore backed by LMDB. The path to the database is optional. If not provided,
   * the database will be stored in a temporary location and be deleted when the process exists.
   *
   * The `rollupAddress` passed is checked against what is stored in the database. If they do not match,
   * the database is cleared before returning the store. This way data is not accidentally shared between
   * different rollup instances.
   *
   * @param rollupAddress - The ETH address of the rollup contract
   * @param path - A path on the disk to store the database. Optional
   * @param log - A logger to use. Optional
   * @returns The store
   */
  static async open(
    rollupAddress: EthAddress,
    path?: string,
    log = createDebugLogger('aztec:kv-store:lmdb'),
  ): Promise<AztecLmdbStore> {
    log.info(`Opening LMDB database at ${path || 'temporary location'}`);

    const rootDb = open({
      path,
    });

    const db = new AztecLmdbStore(rootDb, log);
    await db.#init(rollupAddress);

    return db;
  }

  static openTmp(): Promise<AztecLmdbStore> {
    return AztecLmdbStore.open(EthAddress.random());
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

  async #init(rollupAddress: EthAddress): Promise<void> {
    const storedRollupAddress = this.#rollupAddress.get();
    const rollupAddressString = rollupAddress.toString();

    if (typeof storedRollupAddress === 'string' && rollupAddressString !== storedRollupAddress) {
      this.#log.warn(
        `Rollup address mismatch: expected ${rollupAddress}, found ${storedRollupAddress}. Clearing entire database...`,
      );
      await this.#rootDb.clearAsync();
    }

    await this.#rollupAddress.set(rollupAddressString);
  }
}
