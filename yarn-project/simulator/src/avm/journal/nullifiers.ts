import { siloNullifier } from '@aztec/circuits.js/hash';
import { Fr } from '@aztec/foundation/fields';

import type { CommitmentsDB } from '../../index.js';

/**
 * A class to manage new nullifier staging and existence checks during a contract call's AVM simulation.
 * Maintains a nullifier cache, and ensures that existence checks fall back to the correct source.
 * When a contract call completes, its cached nullifier set can be merged into its parent's.
 */
export class Nullifiers {
  /** Cached nullifiers. */
  private cache: NullifierCache;
  /** Parent's nullifier cache. Checked on cache-miss. */
  private readonly parentCache: NullifierCache | undefined;
  /** Reference to node storage. Checked on parent cache-miss. */
  private readonly hostNullifiers: CommitmentsDB;

  constructor(hostNullifiers: CommitmentsDB, parent?: Nullifiers) {
    this.hostNullifiers = hostNullifiers;
    this.parentCache = parent?.cache;
    this.cache = new NullifierCache();
  }

  /**
   * Get a nullifier's existence status.
   * 1. Check cache.
   * 2. Check parent's cache.
   * 3. Fall back to the host state.
   * 4. Not found! Nullifier does not exist.
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param nullifier - the nullifier to check for
   * @returns exists: whether the nullifier exists at all,
   *          isPending: whether the nullifier was found in a cache,
   *          leafIndex: the nullifier's leaf index if it exists and is not pending (comes from host state).
   */
  public async checkExists(
    storageAddress: Fr,
    nullifier: Fr,
  ): Promise<[/*exists=*/ boolean, /*isPending=*/ boolean, /*leafIndex=*/ Fr]> {
    // First check this cache
    let existsAsPending = this.cache.exists(storageAddress, nullifier);
    // Then check parent's cache
    if (!existsAsPending && this.parentCache) {
      existsAsPending = this.parentCache?.exists(storageAddress, nullifier);
    }
    // Finally try the host's Aztec state (a trip to the database)
    // If the value is found in the database, it will be associated with a leaf index!
    let leafIndex: bigint | undefined = undefined;
    if (!existsAsPending) {
      // silo the nullifier before checking for its existence in the host
      leafIndex = await this.hostNullifiers.getNullifierIndex(siloNullifier(storageAddress, nullifier));
    }
    const exists = existsAsPending || leafIndex !== undefined;
    leafIndex = leafIndex === undefined ? BigInt(0) : leafIndex;
    return Promise.resolve([exists, existsAsPending, new Fr(leafIndex)]);
  }

  /**
   * Stage a new nullifier (append it to the cache).
   *
   * @param storageAddress - the address of the contract that the nullifier is associated with
   * @param nullifier - the nullifier to stage
   */
  public async append(storageAddress: Fr, nullifier: Fr) {
    const [exists, ,] = await this.checkExists(storageAddress, nullifier);
    if (exists) {
      throw new NullifierCollisionError(
        `Nullifier ${nullifier} at contract ${storageAddress} already exists in parent cache or host.`,
      );
    }
    this.cache.append(storageAddress, nullifier);
  }

  /**
   * Merges another nullifier cache into this one.
   *
   * @param incomingNullifiers - the incoming cached nullifiers to merge into this instance's
   */
  public acceptAndMerge(incomingNullifiers: Nullifiers) {
    this.cache.acceptAndMerge(incomingNullifiers.cache);
  }
}

/**
 * A class to cache nullifiers created during a contract call's AVM simulation.
 * "append" updates a map, "exists" checks that map.
 * An instance of this class can merge another instance's cached nullifiers into its own.
 */
export class NullifierCache {
  /**
   * Map for staging nullifiers.
   * One inner-set per contract storage address,
   * each entry being a nullifier.
   */
  private cachePerContract: Map<bigint, Set<bigint>> = new Map();

  /**
   * Check whether a nullifier exists in the cache.
   *
   * @param storageAddress - the address of the contract that the nullifier is associated with
   * @param nullifier - the nullifier to check existence of
   * @returns whether the nullifier is found in the cache
   */
  public exists(storageAddress: Fr, nullifier: Fr): boolean {
    const exists = this.cachePerContract.get(storageAddress.toBigInt())?.has(nullifier.toBigInt());
    return exists ? true : false;
  }

  /**
   * Stage a new nullifier (append it to the cache).
   *
   * @param storageAddress - the address of the contract that the nullifier is associated with
   * @param nullifier - the nullifier to stage
   */
  public append(storageAddress: Fr, nullifier: Fr) {
    let nullifiersForContract = this.cachePerContract.get(storageAddress.toBigInt());
    // If this contract's nullifier set has no cached nullifiers, create a new Set to store them
    if (!nullifiersForContract) {
      nullifiersForContract = new Set();
      this.cachePerContract.set(storageAddress.toBigInt(), nullifiersForContract);
    }
    if (nullifiersForContract.has(nullifier.toBigInt())) {
      throw new NullifierCollisionError(
        `Nullifier ${nullifier} at contract ${storageAddress} already exists in cache.`,
      );
    }
    nullifiersForContract.add(nullifier.toBigInt());
  }

  /**
   * Merge another cache's nullifiers into this instance's.
   *
   * Cached nullifiers in "incoming" must not collide with any present in "this".
   *
   * In practice, "this" is a parent call's pending nullifiers, and "incoming" is a nested call's.
   *
   * @param incomingNullifiers - the incoming cached nullifiers to merge into this instance's
   */
  public acceptAndMerge(incomingNullifiers: NullifierCache) {
    // Iterate over all contracts with staged writes in the child.
    for (const [incomingAddress, incomingCacheAtContract] of incomingNullifiers.cachePerContract) {
      const thisCacheAtContract = this.cachePerContract.get(incomingAddress);
      if (!thisCacheAtContract) {
        // This contract has no nullifiers cached here
        // so just accept incoming cache as-is for this contract.
        this.cachePerContract.set(incomingAddress, incomingCacheAtContract);
      } else {
        // "Incoming" and "this" both have cached nullifiers for this contract.
        // Merge in incoming nullifiers, erroring if there are any duplicates.
        for (const nullifier of incomingCacheAtContract) {
          if (thisCacheAtContract.has(nullifier)) {
            throw new NullifierCollisionError(
              `Failed to accept child call's nullifiers. Nullifier ${nullifier} already exists at contract ${incomingAddress}.`,
            );
          }
          thisCacheAtContract.add(nullifier);
        }
      }
    }
  }
}

export class NullifierCollisionError extends Error {
  constructor(message: string, ...rest: any[]) {
    super(message, ...rest);
    this.name = 'NullifierCollisionError';
  }
}
