import { CircuitsWasm, Fr } from '@aztec/circuits.js';
import { PackedArguments } from '@aztec/types';

/**
 * A cache for packed arguments during transaction execution.
 */
export class PackedArgsCache {
  private cache: Map<bigint, Fr[]>;

  constructor(initialArguments: PackedArguments[] = [], private wasm: CircuitsWasm) {
    this.cache = new Map();
    for (const initialArg of initialArguments) {
      this.cache.set(initialArg.hash.value, initialArg.args);
    }
  }

  /**
   * Creates a new packed arguments cache.
   * @param initialArguments - The initial arguments to add to the cache.
   * @returns The new packed arguments cache.
   */
  public static async create(initialArguments: PackedArguments[] = []): Promise<PackedArgsCache> {
    const wasm = await CircuitsWasm.get();
    return new PackedArgsCache(initialArguments, wasm);
  }

  /**
   * Unpacks packed arguments.
   * @param hash - The hash of the packed arguments.
   * @returns The unpacked arguments.
   */
  public unpack(hash: Fr): Fr[] {
    if (hash.equals(Fr.zero())) {
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
  public async pack(args: Fr[]): Promise<Fr> {
    if (args.length === 0) {
      return Fr.zero();
    }
    const packedArguments = await PackedArguments.fromArgs(args, this.wasm);
    this.cache.set(packedArguments.hash.value, packedArguments.args);
    return packedArguments.hash;
  }
}
