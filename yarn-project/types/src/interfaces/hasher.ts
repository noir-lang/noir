/**
 * Defines hasher interface used by Merkle trees.
 */
export interface Hasher {
  /**
   * Compresses two 32-byte hashes.
   * @param lhs - The first hash.
   * @param rhs - The second hash.
   * @returns The new 32-byte hash.
   */
  compress(lhs: Uint8Array, rhs: Uint8Array): Buffer;

  /**
   * Compresses an array of buffers.
   * @param inputs - The array of buffers to compress.
   * @returns The resulting 32-byte hash.
   */
  compressInputs(inputs: Buffer[]): Buffer;

  /**
   * Get a 32-byte hash from a buffer.
   * @param data - The data buffer.
   * @returns The resulting hash buffer.
   */
  hashToField(data: Uint8Array): Buffer;

  /**
   * Given a buffer containing 32 byte leaves, return a new buffer containing the leaves and all pairs of
   * nodes that define a merkle tree.
   *
   * E.g.
   * Input:  [1][2][3][4]
   * Output: [1][2][3][4][compress(1,2)][compress(3,4)][compress(5,6)].
   *
   * @param leaves - The 32 byte leaves.
   * @returns A tree represented by an array.
   */
  hashToTree(leaves: Buffer[]): Promise<Buffer[]>;
}
