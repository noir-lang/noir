import { AztecAddress } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import type { PublicStateDB } from '../../index.js';

type PublicStorageReadResult = {
  value: Fr;
  exists: boolean;
  cached: boolean;
};

/**
 * A class to manage public storage reads and writes during a contract call's AVM simulation.
 * Maintains a storage write cache, and ensures that reads fall back to the correct source.
 * When a contract call completes, its storage cache can be merged into its parent's.
 */
export class PublicStorage {
  /** Cached storage writes. */
  private readonly cache: PublicStorageCache;

  constructor(
    /** Reference to node storage. Checked on parent cache-miss. */
    private readonly hostPublicStorage: PublicStateDB,
    /** Parent's storage. Checked on this' cache-miss. */
    private readonly parent?: PublicStorage,
  ) {
    this.cache = new PublicStorageCache();
  }

  /**
   * Create a new public storage manager forked from this one
   */
  public fork() {
    return new PublicStorage(this.hostPublicStorage, this);
  }

  /**
   * Get the pending storage.
   */
  public getCache() {
    return this.cache;
  }

  /**
   * Read a storage value from this' cache or parent's (recursively).
   * DOES NOT CHECK HOST STORAGE!
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param slot - the slot in the contract's storage being read from
   * @returns value: the latest value written according to this cache or the parent's. undefined on cache miss.
   */
  public readHereOrParent(storageAddress: Fr, slot: Fr): Fr | undefined {
    // First try check this storage cache
    let value = this.cache.read(storageAddress, slot);
    // Then try parent's storage cache
    if (!value && this.parent) {
      // Note: this will recurse to grandparent/etc until a cache-hit is encountered.
      value = this.parent.readHereOrParent(storageAddress, slot);
    }
    return value;
  }

  /**
   * Read a value from storage.
   * 1. Check cache.
   * 2. Check parent cache.
   * 3. Fall back to the host state.
   * 4. Not found! Value has never been written to before. Flag it as non-existent and return value zero.
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param slot - the slot in the contract's storage being read from
   * @returns exists: whether the slot has EVER been written to before, value: the latest value written to slot, or 0 if never written to before
   */
  public async read(storageAddress: Fr, slot: Fr): Promise<PublicStorageReadResult> {
    let cached = false;
    // Check this cache and parent's (recursively)
    let value = this.readHereOrParent(storageAddress, slot);
    // Finally try the host's Aztec state (a trip to the database)
    if (!value) {
      value = await this.hostPublicStorage.storageRead(storageAddress, slot);
      // TODO(dbanks12): if value retrieved from host storage, we can cache it here
      // any future reads to the same slot can read from cache instead of more expensive
      // DB access
    } else {
      cached = true;
    }
    // if value is undefined, that means this slot has never been written to!
    const exists = value !== undefined;
    const valueOrZero = exists ? value : Fr.ZERO;
    return Promise.resolve({ value: valueOrZero, exists, cached });
  }

  /**
   * Stage a storage write.
   *
   * @param storageAddress - the address of the contract whose storage is being written to
   * @param slot - the slot in the contract's storage being written to
   * @param value - the value being written to the slot
   */
  public write(storageAddress: Fr, slot: Fr, value: Fr) {
    this.cache.write(storageAddress, slot, value);
  }

  /**
   * Merges another PublicStorage's cache (pending writes) into this one.
   *
   * @param incomingPublicStorage - the incoming public storage to merge into this instance's
   */
  public acceptAndMerge(incomingPublicStorage: PublicStorage) {
    this.cache.acceptAndMerge(incomingPublicStorage.cache);
  }

  /**
   * Commits ALL staged writes to the host's state.
   */
  public async commitToDB() {
    for (const [storageAddress, cacheAtContract] of this.cache.cachePerContract) {
      for (const [slot, value] of cacheAtContract) {
        await this.hostPublicStorage.storageWrite(AztecAddress.fromBigInt(storageAddress), new Fr(slot), value);
      }
    }
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
  // FIXME: storage ^ should be private, but its value is used in commitToDB

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
