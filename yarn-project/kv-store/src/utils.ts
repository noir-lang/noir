import { type EthAddress } from '@aztec/foundation/eth-address';
import { type Logger, createDebugLogger } from '@aztec/foundation/log';

import { type AztecKVStore } from './interfaces/store.js';
import { AztecLmdbStore } from './lmdb/store.js';

export function createStore(
  config: { dataDirectory: string | undefined } | (string | undefined),
  rollupAddress: EthAddress,
  log: Logger = createDebugLogger('aztec:kv-store'),
) {
  const dataDirectory = typeof config === 'string' ? config : config?.dataDirectory;
  log.info(dataDirectory ? `Creating data store at directory ${dataDirectory}` : 'Creating ephemeral data store');
  return initStoreForRollup(AztecLmdbStore.open(dataDirectory, false), rollupAddress, log);
}

/**
 * Clears the store if the rollup address does not match the one stored in the database.
 * This is to prevent data from being accidentally shared between different rollup instances.
 * @param store - The store to check
 * @param rollupAddress - The ETH address of the rollup contract
 * @returns A promise that resolves when the store is cleared, or rejects if the rollup address does not match
 */
export async function initStoreForRollup<T extends AztecKVStore>(
  store: T,
  rollupAddress: EthAddress,
  log?: Logger,
): Promise<T> {
  if (!rollupAddress) {
    throw new Error('Rollup address is required');
  }
  const rollupAddressValue = store.openSingleton<ReturnType<EthAddress['toString']>>('rollupAddress');
  const rollupAddressString = rollupAddress.toString();
  const storedRollupAddressString = rollupAddressValue.get();

  if (typeof storedRollupAddressString !== 'undefined' && storedRollupAddressString !== rollupAddressString) {
    log?.warn(`Rollup address mismatch. Clearing entire database...`, {
      expected: rollupAddressString,
      found: storedRollupAddressString,
    });

    await store.clear();
  }

  await rollupAddressValue.set(rollupAddressString);
  return store;
}

/**
 * Opens a temporary store for testing purposes.
 * @param ephemeral - true if the store should only exist in memory and not automatically be flushed to disk. Optional
 * @returns A new store
 */
export function openTmpStore(ephemeral: boolean = false): AztecKVStore {
  return AztecLmdbStore.open(undefined, ephemeral);
}
