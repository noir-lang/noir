import { Fr } from '@aztec/foundation/fields';

import { type MerkleTree } from './merkle_tree.js';
import { MerkleTreeCalculator } from './merkle_tree_calculator.js';
import { computeRootFromSiblingPath } from './sibling_path.js';

describe('sibling path', () => {
  let tree: MerkleTree;

  beforeAll(() => {
    const calculator = new MerkleTreeCalculator(4);
    const leaves = Array.from({ length: 5 }).map((_, i) => new Fr(i).toBuffer());
    tree = calculator.computeTree(leaves);
  });

  test.each([0, 1, 2, 3, 4, 5, 6, 7])('recovers the root from a leaf at index %s and its sibling path', index => {
    const leaf = tree.leaves[index];
    const siblingPath = tree.getSiblingPath(index);
    expect(computeRootFromSiblingPath(leaf, siblingPath, index)).toEqual(tree.root);
  });
});
