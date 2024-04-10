import { PackedValues } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';

/**
 * A cache for packed values (arguments, returns) during transaction execution.
 */
export class PackedValuesCache {
  private cache: Map<bigint, Fr[]>;

  constructor(initialArguments: PackedValues[] = []) {
    this.cache = new Map();
    for (const initialArg of initialArguments) {
      this.cache.set(initialArg.hash.toBigInt(), initialArg.values);
    }
  }

  /**
   * Creates a new packed values cache.
   * @param initialArguments - The initial arguments to add to the cache.
   * @returns The new packed values cache.
   */
  public static create(initialArguments: PackedValues[] = []) {
    return new PackedValuesCache(initialArguments);
  }

  /**
   * Unpacks packed values.
   * @param hash - The hash of the packed values.
   * @returns The unpacked values.
   */
  public unpack(hash: Fr): Fr[] {
    if (hash.equals(Fr.ZERO)) {
      return [];
    }
    const packedValues = this.cache.get(hash.value);
    if (!packedValues) {
      throw new Error(`Packed values for hash ${hash.toString()} not found in cache`);
    }
    return packedValues;
  }

  /**
   * Packs values.
   * @param values - The values to pack.
   * @returns The hash of the packed values.
   */
  public pack(values: Fr[]) {
    if (values.length === 0) {
      return Fr.ZERO;
    }
    const packedValues = PackedValues.fromValues(values);
    this.cache.set(packedValues.hash.value, packedValues.values);
    return packedValues.hash;
  }
}
