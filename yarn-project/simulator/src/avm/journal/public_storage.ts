import { Fr } from '@aztec/foundation/fields';

import type { PublicStateDB } from '../../index.js';

/**
 * A class to manage public storage reads and writes during a contract call's AVM simulation.
 * Maintains a storage write cache, and ensures that reads fall back to the correct source.
 * When a contract call completes, its storage cache can be merged into its parent's.
 */
export class PublicStorage {
  /** Cached storage writes. */
  private cache: PublicStorageCache;
  /** Parent's storage cache. Checked on cache-miss. */
  private readonly parentCache: PublicStorageCache | undefined;
  /** Reference to node storage. Checked on parent cache-miss. */
  private readonly hostPublicStorage: PublicStateDB;

  constructor(hostPublicStorage: PublicStateDB, parent?: PublicStorage) {
    this.hostPublicStorage = hostPublicStorage;
    this.parentCache = parent?.cache;
    this.cache = new PublicStorageCache();
  }

  /**
   * Get the pending storage.
   */
  public getCache() {
    return this.cache;
  }

  /**
   * Read a value from storage.
   * 1. Check cache.
   * 2. Check parent's cache.
   * 3. Fall back to the host state.
   * 4. Not found! Value has never been written to before. Flag it as non-existent and return value zero.
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param slot - the slot in the contract's storage being read from
   * @returns exists: whether the slot has EVER been written to before, value: the latest value written to slot, or 0 if never written to before
   */
  public async read(storageAddress: Fr, slot: Fr): Promise<[/*exists=*/ boolean, /*value=*/ Fr]> {
    // First try check this storage cache
    let value = this.cache.read(storageAddress, slot);
    // Then try parent's storage cache (if it exists / written to earlier in this TX)
    if (!value && this.parentCache) {
      value = this.parentCache?.read(storageAddress, slot);
    }
    // Finally try the host's Aztec state (a trip to the database)
    if (!value) {
      value = await this.hostPublicStorage.storageRead(storageAddress, slot);
    }
    // if value is undefined, that means this slot has never been written to!
    const exists = value !== undefined;
    const valueOrZero = exists ? value : Fr.ZERO;
    return Promise.resolve([exists, valueOrZero]);
  }

  /**
   * Stage a storage write.
   *
   * @param storageAddress - the address of the contract whose storage is being written to
   * @param slot - the slot in the contract's storage being written to
   * @param value - the value being written to the slot
   */
  public write(storageAddress: Fr, key: Fr, value: Fr) {
    this.cache.write(storageAddress, key, value);
  }

  /**
   * Merges another PublicStorage's cache (pending writes) into this one.
   *
   * @param incomingPublicStorage - the incoming public storage to merge into this instance's
   */
  public acceptAndMerge(incomingPublicStorage: PublicStorage) {
    this.cache.acceptAndMerge(incomingPublicStorage.cache);
  }
}

/**
 * A class to cache writes to public storage during a contract call's AVM simulation.
 * "Writes" update a map, "reads" check that map or return undefined.
 * An instance of this class can merge another instance's staged writes into its own.
 */
class PublicStorageCache {
  /**
   * Map for staging storage writes.
   * One inner-map per contract storage address,
   * mapping storage slot to latest staged write value.
   */
  public cachePerContract: Map<bigint, Map<bigint, Fr>> = new Map();
  // FIXME: storage ^ should be private, but its value is used in tests for "currentStorageValue"

  /**
   * Read a staged value from storage, if it has been previously written to.
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param slot - the slot in the contract's storage being read from
   * @returns the latest value written to slot, or undefined if no value has been written
   */
  public read(storageAddress: Fr, slot: Fr): Fr | undefined {
    return this.cachePerContract.get(storageAddress.toBigInt())?.get(slot.toBigInt());
  }

  /**
   * Stage a storage write.
   *
   * @param storageAddress - the address of the contract whose storage is being written to
   * @param slot - the slot in the contract's storage being written to
   * @param value - the value being written to the slot
   */
  public write(storageAddress: Fr, slot: Fr, value: Fr) {
    let cacheAtContract = this.cachePerContract.get(storageAddress.toBigInt());
    if (!cacheAtContract) {
      // If this contract's storage has no staged modifications, create a new inner map to store them
      cacheAtContract = new Map();
      this.cachePerContract.set(storageAddress.toBigInt(), cacheAtContract);
    }
    cacheAtContract.set(slot.toBigInt(), value);
  }

  /**
   * Merges another cache's staged writes into this instance's cache.
   *
   * Staged modifications in "incoming" take precedence over those
   * present in "this" as they are assumed to occur after this' writes.
   *
   * In practice, "this" is a parent call's storage cache, and "incoming" is a nested call's.
   *
   * @param incomingStorageCache - the incoming storage write cache to merge into this instance's
   */
  public acceptAndMerge(incomingStorageCache: PublicStorageCache) {
    // Iterate over all incoming contracts with staged writes.
    for (const [incomingAddress, incomingCacheAtContract] of incomingStorageCache.cachePerContract) {
      const thisCacheAtContract = this.cachePerContract.get(incomingAddress);
      if (!thisCacheAtContract) {
        // The contract has no storage writes staged here
        // so just accept the incoming cache as-is for this contract.
        this.cachePerContract.set(incomingAddress, incomingCacheAtContract);
      } else {
        // "Incoming" and "this" both have staged writes for this contract.
        // Merge in incoming staged writes, giving them precedence over this'.
        for (const [slot, value] of incomingCacheAtContract) {
          thisCacheAtContract.set(slot, value);
        }
      }
    }
  }
}
