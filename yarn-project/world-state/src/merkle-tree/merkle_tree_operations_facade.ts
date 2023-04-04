import { SiblingPath } from '@aztec/merkle-tree';
import { LeafData, MerkleTreeDbOperations, MerkleTreeId, MerkleTreeOperations, TreeInfo } from '../index.js';

/**
 * Wraps a MerkleTreeDbOperations to call all functions with a preset includeUncommitted flag.
 */
export class MerkleTreeOperationsFacade implements MerkleTreeOperations {
  constructor(private trees: MerkleTreeDbOperations, private includeUncommitted: boolean) {}
  getTreeInfo(treeId: MerkleTreeId): Promise<TreeInfo> {
    return this.trees.getTreeInfo(treeId, this.includeUncommitted);
  }
  appendLeaves(treeId: MerkleTreeId, leaves: Buffer[]): Promise<void> {
    return this.trees.appendLeaves(treeId, leaves);
  }
  getSiblingPath(treeId: MerkleTreeId, index: bigint): Promise<SiblingPath> {
    return this.trees.getSiblingPath(treeId, index, this.includeUncommitted);
  }
  getPreviousValueIndex(
    treeId: MerkleTreeId.NULLIFIER_TREE,
    value: bigint,
  ): Promise<{ index: number; alreadyPresent: boolean }> {
    return this.trees.getPreviousValueIndex(treeId, value, this.includeUncommitted);
  }
  getLeafData(treeId: MerkleTreeId.NULLIFIER_TREE, index: number): Promise<LeafData | undefined> {
    return this.trees.getLeafData(treeId, index, this.includeUncommitted);
  }
  findLeafIndex(treeId: MerkleTreeId, value: Buffer): Promise<bigint | undefined> {
    return this.trees.findLeafIndex(treeId, value, this.includeUncommitted);
  }
  getLeafValue(treeId: MerkleTreeId, index: bigint): Promise<Buffer | undefined> {
    return this.trees.getLeafValue(treeId, index, this.includeUncommitted);
  }
}
