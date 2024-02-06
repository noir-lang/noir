import { PackedArguments } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';

/**
 * A cache for packed arguments during transaction execution.
 */
export class PackedArgsCache {
  private cache: Map<bigint, Fr[]>;

  constructor(initialArguments: PackedArguments[] = []) {
    this.cache = new Map();
    for (const initialArg of initialArguments) {
      this.cache.set(initialArg.hash.toBigInt(), initialArg.args);
    }
  }

  /**
   * Creates a new packed arguments cache.
   * @param initialArguments - The initial arguments to add to the cache.
   * @returns The new packed arguments cache.
   */
  public static create(initialArguments: PackedArguments[] = []) {
    return new PackedArgsCache(initialArguments);
  }

  /**
   * Unpacks packed arguments.
   * @param hash - The hash of the packed arguments.
   * @returns The unpacked arguments.
   */
  public unpack(hash: Fr): Fr[] {
    if (hash.equals(Fr.ZERO)) {
      return [];
    }
    const packedArgs = this.cache.get(hash.value);
    if (!packedArgs) {
      throw new Error(`Packed arguments for hash ${hash.toString()} not found in cache`);
    }
    return packedArgs;
  }

  /**
   * Packs arguments.
   * @param args - The arguments to pack.
   * @returns The hash of the packed arguments.
   */
  public pack(args: Fr[]) {
    if (args.length === 0) {
      return Fr.ZERO;
    }
    const packedArguments = PackedArguments.fromArgs(args);
    this.cache.set(packedArguments.hash.value, packedArguments.args);
    return packedArguments.hash;
  }
}
