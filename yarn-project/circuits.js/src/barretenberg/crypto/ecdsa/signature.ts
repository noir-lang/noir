import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { randomBytes } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { mapTuple } from '@aztec/foundation/serialize';

import { type Signature } from '../signature/index.js';

/**
 * ECDSA signature used for transactions.
 * @see cpp/barretenberg/cpp/src/barretenberg/crypto/ecdsa/ecdsa.hpp
 */
export class EcdsaSignature implements Signature {
  constructor(
    /**
     * The r byte-array (32 bytes) in an ECDSA signature.
     */
    public r: Buffer,
    /**
     * The s byte-array (32 bytes) in an ECDSA signature.
     */
    public s: Buffer,
    /**
     * The recovery id (1 byte) in an ECDSA signature.
     */
    public v: Buffer,
  ) {
    if (r.length != 32) {
      throw new Error(`Invalid length of 'r' in ECDSA signature. Expected 32, got ${s.length}`);
    }
    if (s.length != 32) {
      throw new Error(`Invalid length of 's' in ECDSA signature. Expected 32, got ${r.length}`);
    }
    if (v.length != 1) {
      throw new Error(`Invalid length of 'v' in ECDSA signature. Expected 1, got ${v.length}`);
    }
  }

  /**
   * Converts an ECDSA signature to a buffer.
   * @returns A buffer.
   */
  toBuffer() {
    return Buffer.concat([this.r, this.s, this.v]);
  }

  /**
   * Deserializes the signature from a buffer.
   * @param buffer - The buffer from which to deserialize the signature.
   * @returns The ECDSA signature
   */
  public static fromBuffer(buffer: Buffer) {
    return new EcdsaSignature(buffer.subarray(0, 32), buffer.subarray(32, 64), buffer.subarray(64, 65));
  }

  /**
   * Creates a new instance from bigint r and s values.
   * @param r - r.
   * @param s - s.
   * @param v - v.
   * @returns The resulting signature.
   */
  public static fromBigInts(r: bigint, s: bigint, v: number) {
    return new EcdsaSignature(toBufferBE(r, 32), toBufferBE(s, 32), Buffer.from([v]));
  }

  /**
   * Generate a random ECDSA signature for testing.
   * @returns A randomly generated ECDSA signature (not a valid one).
   */
  public static random() {
    return new EcdsaSignature(randomBytes(32), randomBytes(32), Buffer.from([27]));
  }

  /**
   * Convert an ECDSA signature to a buffer.
   * @returns A 65-character string of the form 0x<r><s><v>.
   */
  toString() {
    return `0x${this.toBuffer().toString('hex')}`;
  }

  /**
   * Converts the signature to an array of fields.
   * @param includeV - Determines whether the 'v' term is included
   * @returns The signature components as an array of fields
   */
  toFields(includeV = false): Fr[] {
    const sig = this.toBuffer();

    const buf1 = Buffer.alloc(32);
    const buf2 = Buffer.alloc(32);
    const buf3 = Buffer.alloc(32);

    sig.copy(buf1, 1, 0, 31);
    sig.copy(buf2, 1, 31, 62);
    sig.copy(buf3, 1, 62, includeV ? 65 : 64);

    return mapTuple([buf1, buf2, buf3], Fr.fromBuffer);
  }
}
