import { createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';
import { SiblingPath } from '@aztec/types/membership';

import { randomBytes } from 'crypto';

import { INITIAL_LEAF, newTree } from '../index.js';
import { UpdateOnlyTree } from '../interfaces/update_only_tree.js';
import { loadTree } from '../load_tree.js';
import { Pedersen } from '../pedersen.js';
import { standardBasedTreeTestSuite } from '../test/standard_based_test_suite.js';
import { treeTestSuite } from '../test/test_suite.js';
import { SparseTree } from './sparse_tree.js';

const log = createDebugLogger('aztec:sparse_tree_test');

const createDb = async (db: AztecKVStore, hasher: Hasher, name: string, depth: number): Promise<UpdateOnlyTree> => {
  return await newTree(SparseTree, db, hasher, name, depth);
};

const createFromName = async (db: AztecKVStore, hasher: Hasher, name: string): Promise<UpdateOnlyTree> => {
  return await loadTree(SparseTree, db, hasher, name);
};

const TEST_TREE_DEPTH = 3;

treeTestSuite('SparseTree', createDb, createFromName);
standardBasedTreeTestSuite('SparseTree', createDb);

describe('SparseTreeSpecific', () => {
  let pedersen: Pedersen;

  beforeEach(() => {
    pedersen = new Pedersen();
  });

  it('throws when index is bigger than (2^DEPTH - 1) ', async () => {
    const db = await AztecLmdbStore.openTmp();
    const depth = 32;
    const tree = await createDb(db, pedersen, 'test', depth);

    const index = 2n ** BigInt(depth);
    expect(() => tree.updateLeaf(Buffer.alloc(32), index)).toThrow();
  });

  it('updating non-empty leaf does not change tree size', async () => {
    const depth = 32;
    const maxIndex = 2 ** depth - 1;

    const db = await AztecLmdbStore.openTmp();
    const tree = await createDb(db, pedersen, 'test', depth);

    const randomIndex = BigInt(Math.floor(Math.random() * maxIndex));
    expect(tree.getNumLeaves(false)).toEqual(0n);

    // Insert a leaf
    await tree.updateLeaf(randomBytes(32), randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(1n);

    // Update a leaf
    await tree.updateLeaf(randomBytes(32), randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(1n);
  });

  it('deleting leaf decrements tree size', async () => {
    const depth = 254;
    const maxIndex = 2 ** depth - 1;

    const db = await AztecLmdbStore.openTmp();
    const tree = await createDb(db, pedersen, 'test', depth);

    const randomIndex = BigInt(Math.floor(Math.random() * maxIndex));
    expect(tree.getNumLeaves(false)).toEqual(0n);

    // Insert a leaf
    await tree.updateLeaf(randomBytes(32), randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(1n);

    // Delete a leaf
    await tree.updateLeaf(INITIAL_LEAF, randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(0n);
  });

  it('should have correct root and sibling path after in a "non-append-only" way', async () => {
    const db = await AztecLmdbStore.openTmp();
    const tree = await createDb(db, pedersen, 'test', 3);

    const level2ZeroHash = pedersen.hash(INITIAL_LEAF, INITIAL_LEAF);
    const level1ZeroHash = pedersen.hash(level2ZeroHash, level2ZeroHash);

    expect(tree.getNumLeaves(false)).toEqual(0n);
    expect(tree.getRoot(false)).toEqual(pedersen.hash(level1ZeroHash, level1ZeroHash));

    // Insert leaf at index 3
    let level1LeftHash: Buffer;
    const leafAtIndex3 = randomBytes(32);
    {
      await tree.updateLeaf(leafAtIndex3, 3n);
      expect(tree.getNumLeaves(true)).toEqual(1n);
      const level2Hash = pedersen.hash(INITIAL_LEAF, leafAtIndex3);
      level1LeftHash = pedersen.hash(level2ZeroHash, level2Hash);
      const root = pedersen.hash(level1LeftHash, level1ZeroHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(3n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level2ZeroHash, level1ZeroHash]),
      );
    }

    // Insert leaf at index 6
    let level1RightHash: Buffer;
    {
      const leafAtIndex6 = randomBytes(32);
      await tree.updateLeaf(leafAtIndex6, 6n);
      expect(tree.getNumLeaves(true)).toEqual(2n);
      const level2Hash = pedersen.hash(leafAtIndex6, INITIAL_LEAF);
      level1RightHash = pedersen.hash(level2ZeroHash, level2Hash);
      const root = pedersen.hash(level1LeftHash, level1RightHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(6n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level2ZeroHash, level1LeftHash]),
      );
    }

    // Insert leaf at index 2
    const leafAtIndex2 = randomBytes(32);
    {
      await tree.updateLeaf(leafAtIndex2, 2n);
      expect(tree.getNumLeaves(true)).toEqual(3n);
      const level2Hash = pedersen.hash(leafAtIndex2, leafAtIndex3);
      level1LeftHash = pedersen.hash(level2ZeroHash, level2Hash);
      const root = pedersen.hash(level1LeftHash, level1RightHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(2n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [leafAtIndex3, level2ZeroHash, level1RightHash]),
      );
    }

    // Updating leaf at index 3
    {
      const updatedLeafAtIndex3 = randomBytes(32);
      await tree.updateLeaf(updatedLeafAtIndex3, 3n);
      expect(tree.getNumLeaves(true)).toEqual(3n);
      const level2Hash = pedersen.hash(leafAtIndex2, updatedLeafAtIndex3);
      level1LeftHash = pedersen.hash(level2ZeroHash, level2Hash);
      const root = pedersen.hash(level1LeftHash, level1RightHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(3n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [leafAtIndex2, level2ZeroHash, level1RightHash]),
      );
    }
  });

  // This one is a performance measurement and is enabled only to check regression in performance.
  it.skip('measures time of inserting 1000 leaves at random positions for depth 254', async () => {
    const depth = 254;
    const maxIndex = 2 ** depth - 1;

    const db = await AztecLmdbStore.openTmp();
    const tree = await createDb(db, pedersen, 'test', depth);

    const leaves = Array.from({ length: 1000 }).map(() => randomBytes(32));
    const indices = Array.from({ length: 1000 }).map(() => BigInt(Math.floor(Math.random() * maxIndex)));

    const start = Date.now();
    await Promise.all(leaves.map((leaf, i) => tree.updateLeaf(leaf, indices[i])));
    const end = Date.now();
    log(`Inserting 1000 leaves at random positions for depth 254 took ${end - start}ms`);
  }, 300_000);
});
