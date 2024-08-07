import { type MerkleTreeId } from '@aztec/circuit-types';
import { type Fr } from '@aztec/circuits.js';
import { type AppendOnlyTree, type IndexedTree } from '@aztec/merkle-tree';

export type MerkleTreeMap = {
  [MerkleTreeId.NULLIFIER_TREE]: IndexedTree;
  [MerkleTreeId.NOTE_HASH_TREE]: AppendOnlyTree<Fr>;
  [MerkleTreeId.PUBLIC_DATA_TREE]: IndexedTree;
  [MerkleTreeId.L1_TO_L2_MESSAGE_TREE]: AppendOnlyTree<Fr>;
  [MerkleTreeId.ARCHIVE]: AppendOnlyTree<Fr>;
};
