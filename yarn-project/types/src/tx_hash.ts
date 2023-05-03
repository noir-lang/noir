import { toBigInt } from '@aztec/foundation/serialize';

/**
 * A class representing hash of Aztec transaction.
 */
export class TxHash {
  /**
   * The size of the hash in bytes.
   */
  public static SIZE = 32;

  constructor(
    /**
     * The buffer containing the hash.
     */
    public readonly buffer: Buffer,
  ) {}

  /**
   * Checks if this hash and another hash are equal.
   * @param hash - A hash to compare with.
   * @returns True if the hashes are equal, false otherwise.
   */
  public equals(hash: TxHash): boolean {
    return this.buffer.equals(hash.buffer);
  }

  /**
   * Convert this hash to a hex string.
   * @returns The hex string.
   */
  public toString() {
    return this.buffer.toString('hex');
  }
  /**
   * Convert this hash to a big int.
   * @returns The big int.
   */
  public toBigInt() {
    return toBigInt(this.buffer);
  }
}
