import { Pedersen } from '../../index.js';

/**
 * A test utility allowing us to count the number of times the hash function has been called.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export class PedersenWithCounter extends Pedersen {
  /**
   * The number of times the hash function has been called.
   */
  public hashCounter = 0;

  /**
   * Hashes two 32-byte arrays.
   * @param lhs - The first 32-byte array.
   * @param rhs - The second 32-byte array.
   * @returns The new 32-byte hash.
   * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
   * purposes.
   */
  public override hash(lhs: Uint8Array, rhs: Uint8Array): Buffer {
    this.hashCounter++;
    return super.hash(lhs, rhs);
  }

  /**
   * Resets the hash counter.
   * @returns void
   */
  public resetCounter() {
    this.hashCounter = 0;
  }
}
