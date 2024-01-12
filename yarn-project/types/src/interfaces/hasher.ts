/**
 * Defines hasher interface used by Merkle trees.
 */
export interface Hasher {
  /**
   * Hash two arrays.
   * @param lhs - The first array.
   * @param rhs - The second array.
   * @returns The new 32-byte hash.
   */
  hash(lhs: Uint8Array, rhs: Uint8Array): Buffer;

  /**
   * Hashes an array of buffers.
   * @param inputs - The array of buffers to hash.
   * @returns The resulting 32-byte hash.
   */
  hashInputs(inputs: Buffer[]): Buffer;
}
