import { pedersenHash } from '@aztec/foundation/crypto';
import { type Fr } from '@aztec/foundation/fields';

/**
 * Computes the resulting storage slot for an entry in a map.
 * @param mapSlot - The slot of the map within state.
 * @param key - The key of the map.
 * @returns The slot in the contract storage where the value is stored.
 */
export function deriveStorageSlotInMap(
  mapSlot: Fr | bigint,
  key: {
    /** Convert key to a field. */
    toField: () => Fr;
  },
): Fr {
  return pedersenHash([mapSlot, key.toField()]);
}
