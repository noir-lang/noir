import { SiblingPath } from './sibling_path/sibling_path.js';

/**
 * Defines the interface for a source of sibling paths.
 */
export interface SiblingPathSource {
  getSiblingPath(index: bigint): Promise<SiblingPath>;
}

/**
 * Defines the interface for a Merkle tree.
 */
export interface MerkleTree extends SiblingPathSource {
  getRoot(): Buffer;
  getNumLeaves(): bigint;
  appendLeaves(leaves: Buffer[]): Promise<void>;
  commit(): Promise<void>;
  rollback(): Promise<void>;
  getLeafValue(index: bigint): Promise<Buffer | undefined>;
}
