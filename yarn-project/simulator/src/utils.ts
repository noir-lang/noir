import { pedersenHash } from '@aztec/foundation/crypto';
import { type Fr } from '@aztec/foundation/fields';

/**
 * Computes the resulting storage slot for an entry in a mapping.
 * @param mappingSlot - The slot of the mapping within state.
 * @param key - The key of the mapping.
 * @returns The slot in the contract storage where the value is stored.
 */
export function computeSlotForMapping(
  mappingSlot: Fr,
  key: {
    /** Serialize to a field. */
    toField: () => Fr;
  },
) {
  return pedersenHash([mappingSlot, key.toField()]);
}
