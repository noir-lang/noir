import { BufferReader, Tuple, mapTuple } from '@aztec/foundation/serialize';
import { assertMemberLength } from '../utils/jsUtils.js';
import { Bufferable, serializeToBuffer } from '../utils/serialize.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from './index.js';

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

  toFields(includeV = false): Tuple<Fr, 3> {
    const sig = this.toBuffer();

    const buf1 = Buffer.alloc(32);
    const buf2 = Buffer.alloc(32);
    const buf3 = Buffer.alloc(32);

    sig.copy(buf1, 1, 0, 31);
    sig.copy(buf2, 1, 31, 62);
    sig.copy(buf3, 1, 62, includeV ? 65 : 64);

    return mapTuple([buf1, buf2, buf3], Fr.fromBuffer);
  }

  static fromBigInts(r: bigint, s: bigint, v: number) {
    return new EcdsaSignature(toBufferBE(r, 32), toBufferBE(s, 32), Buffer.from([v]));
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

  static empty(): EcdsaSignature {
    return new EcdsaSignature(Buffer.alloc(32, 0), Buffer.alloc(32, 0), Buffer.alloc(1, 0));
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
