/**
 * Defines hasher interface used by Merkle trees.
 */
export interface Hasher {
  compress(lhs: Uint8Array, rhs: Uint8Array): Buffer;
  hashToField(data: Uint8Array): Buffer;
  hashToTree(leaves: Buffer[]): Promise<Buffer[]>;
}
