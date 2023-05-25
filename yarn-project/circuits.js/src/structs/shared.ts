import { BufferReader } from '@aztec/foundation/serialize';
import { assertMemberLength } from '../utils/jsUtils.js';
import { Bufferable, serializeToBuffer } from '../utils/serialize.js';
import { randomBytes } from '@aztec/foundation/crypto';

/**
 * Implementation of a vector. Matches how we are serializing and deserializing vectors in cpp (length in the first position, followed by the items).
 */
export class Vector<T extends Bufferable> {
  constructor(
    /**
     * Items in the vector.
     */
    public items: T[],
  ) {}

  toBuffer() {
    return serializeToBuffer(this.items.length, this.items);
  }

  toFriendlyJSON() {
    return this.items;
  }
}

/**
 * A type alias for a 32-bit unsigned integer.
 */
export type UInt32 = number;

/**
 * ECDSA signature used for transactions.
 * @see cpp/barretenberg/cpp/src/barretenberg/crypto/ecdsa/ecdsa.hpp
 */
export class EcdsaSignature {
  constructor(
    /**
     * Value `r` of the signature.
     */
    public r: Buffer,
    /**
     * Value `s` of the signature.
     */
    public s: Buffer,
    /**
     * Value `v` of the signature.
     */
    public v: Buffer,
  ) {
    assertMemberLength(this, 'r', 32);
    assertMemberLength(this, 's', 32);
    assertMemberLength(this, 'v', 1);
  }

  toBuffer() {
    return serializeToBuffer(this.r, this.s, this.v);
  }

  static fromBuffer(buffer: Buffer | BufferReader): EcdsaSignature {
    const reader = BufferReader.asReader(buffer);
    return new EcdsaSignature(reader.readBytes(32), reader.readBytes(32), reader.readBytes(1));
  }

  /**
   * Returns a random/placeholder ECDSA signature.
   * @returns A random placeholder ECDSA signature.
   */
  public static random(): EcdsaSignature {
    return new EcdsaSignature(randomBytes(32), randomBytes(32), randomBytes(1));
  }
}

/**
 * Composer prover type.
 */
export enum ComposerType {
  STANDARD = 0,
  TURBO = 1,
  PLOOKUP = 2,
  STANDARD_HONK = 3,
}

/**
 * Rollup types.
 */
export enum RollupTypes {
  Base = 0,
  Merge = 1,
  Root = 2,
}
